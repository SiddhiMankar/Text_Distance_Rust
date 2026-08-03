# Architectural & Design Decisions Log: `textdistancerust`

This document records all architectural choices, trait abstractions, empty-input handling rules, and behavioral tradeoff decisions made during the Rust port of `textdistance`.

---

## 1. Standalone Package Architecture & Isolation
* **Decision**: All Rust implementation code lives exclusively in the `textdistancerust/` workspace directory.
* **Reasoning**: Satisfies `PROJECT_RULES.md` constraint requiring strict separation from the reference Python codebase. The crate compiles standalone (`cargo build --release`) with **zero runtime dependency** on Python, PyO3, or subprocess wrappers.

---

## 2. Generic Sequence Traits (`DistanceMetric<T>` & `SimilarityMetric<T>`)
* **Decision**: Abstract metrics using generic traits over sequence slices `&[T]`.
* **Reasoning**:
  * Python's `textdistance` operates on strings, byte sequences, lists of tokens, and lists of numbers.
  * Hardcoding `&str` in Rust would cause byte-boundary panics on multi-byte UTF-8 Unicode characters and fail on non-string lists.
  * Trait definitions:
    * `DistanceMetric<T>`: Requires `distance()` and `maximum()`. Automatically provides `similarity()` (`maximum - distance`), `normalized_distance()` (`distance / maximum`), and `normalized_similarity()` (`1.0 - normalized_distance`).
    * `SimilarityMetric<T>`: Requires `similarity()` and `maximum()`. Automatically provides `distance()` (`maximum - similarity`), `normalized_distance()` (`distance / maximum`), and `normalized_similarity()` (`similarity / maximum`).

---

## 3. Empty-Input & Zero-Max Parity Convention
* **Decision**: Standardize default trait behavior for zero-maximum cases (`maximum == 0.0`), while enforcing per-algorithm inspection of Python reference code.
* **Reasoning**:
  * When inputs are empty (`("", "")`), `maximum(s1, s2)` evaluates to `0.0`. Dividing by zero in floating-point math yields `NaN`.
  * For standard metrics, `normalized_distance` defaults to `0.0` and `normalized_similarity` defaults to `1.0`.
  * **Rule**: Trait defaults serve as fallbacks only. If an algorithm in Python explicitly handles empty inputs differently (e.g. raising `ZeroDivisionError`, returning `0.0`, or returning `-inf` like `Tanimoto`), the algorithm's Rust struct overrides the default trait method to preserve exact behavioral parity.

---

## 4. Persistent IPC Protocol for Differential Fuzzing
* **Decision**: Implement a persistent stdin/stdout streaming JSON IPC executable (`textdistancerust-cli`).
* **Reasoning**:
  * Process creation overhead on Windows (spawning a process per test case) adds tens of minutes of latency for $10,000+$ fuzz iterations.
  * The Python Hypothesis driver (`fuzz-harness/fuzz_driver.py`) spawns `textdistancerust-cli` once at startup and streams test cases continuously as single-line JSON requests (`{"alg": "...", "s1": "...", "s2": "..."}`) over `stdin`, receiving single-line JSON responses (`{"similarity": ..., "distance": ...}`) from `stdout`.

---

## 5. Tokenizer & Unicode Handling Strategy (`tokenizer.rs`)
* **Decision**: Provide UTF-8 scalar conversion (`to_char_vec`), whitespace word splitting (`to_word_vec`), and generic sliding window $q$-gram generation (`find_ngrams`).
* **Reasoning**:
  * Python string indexing `s[i]` indexes Unicode scalar values (characters), whereas Rust string indexing `&s[i..j]` operates on byte offsets.
  * Converting input string slices `&str` into `Vec<char>` prior to algorithm execution guarantees safe UTF-8 indexing and exact matching with Python sequence indexing.

---

## 6. Subsequence Extraction vs. Numeric Metrics Dual API (`Prefix`, `Postfix`, `LCS` Family)
* **Decision**: Expose both slice-returning helper methods and numeric metric trait implementations for sequence sub-extractors.
* **Reasoning**:
  * In Python `textdistance`, calling `prefix("hello", "help")` (`__call__`) returns the common prefix substring `'hel'`, whereas calling `prefix.similarity("hello", "help")` returns `len(prefix)` (`3`).
  * **Explicit Convention**: Prefix/Postfix's numeric similarity/distance interface is a derived convenience not present in the original API; verified against the original's literal substring output, with numeric fields checked against `len(original_output)` as our own defined convention.
  * In Rust, `Prefix` implements `SimilarityMetric<T>` for numeric distance/similarity calculations (`similarity` = `3.0`, `distance` = `2.0`, `normalized_similarity` = `0.6`, `normalized_distance` = `0.4`), while providing a dedicated slice method `prefix<'a>(&self, s1: &'a [T], s2: &'a [T]) -> &'a [T]` for substring extraction.
  * The differential fuzz harness implements a two-track comparison: verifying the Rust `prefix()` slice against Python's `Prefix()(s1, s2)` string output directly, and checking numeric fields against `len(original_output)`.

---

## 7. Strict Zero `unsafe` Policy
* **Decision**: Enforce `#![forbid(unsafe_code)]` across the entire crate.
* **Reasoning**: Compliance with `PROJECT_RULES.md` hard constraints. Safe Rust iterators (`windows`, `zip`, `split_whitespace`) provide performance without memory safety risks.

---

## 8. `Matrix` Custom Score Map JSON Serialization (`mat`)
* **Decision**: Represent `mat` in JSON request payloads as a list of `[s1, s2, score]` triples (`Vec<(String, String, f64)>`) instead of JSON object keys.
* **Reasoning**:
  * Standard JSON object keys must be plain strings; composite tuple keys like `("cat", "bat")` break standard JSON syntax.
  * Passing `[["cat", "bat", 0.5]]` cleanly sidesteps string key constraints, allowing deserialization into `HashMap<(String, String), f64>` in Rust and seamless generation in the Python Hypothesis driver.

## 9. `Jaccard` $q$-gram Short Input Handling & Upstream Python Crash Protection
* **Decision**: Handle zero-union token sets safely in Rust (`union == 0` $\rightarrow$ `similarity = 0.0`), and document the upstream Python `ZeroDivisionError`.
* **Empirical Python Upstream Bug Verification**:
  ```python
  import textdistance
  textdistance.Jaccard(qval=2)('0', '1')
  ```
  **Traceback**:
  ```text
  Traceback (most recent call last):
    File "<string>", line 1, in <module>
      import textdistance; print(textdistance.Jaccard(qval=2)('0', '1'))
                                 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^
    File "textdistance/algorithms/token_based.py", line 57, in __call__
      return intersection / union
             ~~~~~~~~~~~~~^~~~~~~
  ZeroDivisionError: division by zero
  ```
* **Reasoning**:
  * In Python `textdistance`, when $q$-gram size $q > 1$ and non-equal inputs are shorter than $q$ (e.g. `s1="0"`, `s2="1"`, `qval=2`), `find_ngrams` produces empty token lists `[]`, causing Python's `intersection / union` to raise an unhandled `ZeroDivisionError`.
  * Rust evaluates raw string identity (`req.s1 == req.s2`) first, returning `1.0` for identical inputs (e.g. `("", "")`), and returns `0.0` when token union is empty for distinct inputs, ensuring 100% memory safety and crash resilience.

## 10. `Overlap` Coefficient Implementation & Minimum Count Zero Protection
* **Decision**: Implement Overlap coefficient formula $\frac{|A \cap B|}{\min(|A|, |B|)}$ using multiset/set counts, handling `min_count == 0` safely in Rust (`min_count == 0` $\rightarrow$ `similarity = 0.0`).
* **Empirical Python Upstream Bug Verification**:
  ```python
  import textdistance
  textdistance.Overlap(qval=2)('0', '1')
  ```
  **Traceback**:
  ```text
  Traceback (most recent call last):
    File "<string>", line 1, in <module>
      import textdistance; print(textdistance.Overlap(qval=2)('0', '1'))
                                 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^
    File "textdistance/algorithms/token_based.py", line 179, in __call__
      return intersection / min(sequences)
             ~~~~~~~~~~~~~^~~~~~~~~~~~~~~~
  ZeroDivisionError: division by zero
  ```
* **Edge Case Behavior Analysis**:
  * **Asymmetric Empty Input (`('', '0')`)**: `quick_answer` in Python checks `not all(sequences)` and returns `0.0` cleanly without crashing.
  * **Short Non-Equal Inputs (`('0', '1')` with `qval=2`)**: `quick_answer` returns `None`. $q$-gram tokenization produces empty token lists `[]` (len 0) for both strings, leading to `min(sequences) == 0` and crashing with `ZeroDivisionError` at line 179 of `token_based.py`.
  * **Rust Resilience**: Rust evaluates raw string identity (`req.s1 == req.s2`) first, returning `1.0` for identical inputs (e.g. `("", "")`), and returns `0.0` when `min_count == 0` for non-identical inputs, ensuring 100% memory safety.
* In Python `textdistance`, `Overlap(qval=2)('0', '1')` raises `ZeroDivisionError: division by zero` because `min(sequences)` evaluates to 0 when inputs are shorter than $q$-gram length $q$.
  * Rust evaluates raw string identity (`req.s1 == req.s2`) first, returning `1.0` for identical inputs (e.g. `("", "")`), and returns `0.0` when `min_count == 0` for non-identical inputs.
## 11. `Cosine` Similarity (Ochiai Coefficient) & Zero-Product Protection
* **Decision**: Implement Cosine similarity formula $\frac{|A \cap B|}{\sqrt{|A| \times |B|}}$ using multiset/set counts, handling `prod == 0.0` safely in Rust (`prod == 0.0` $\rightarrow$ `similarity = 0.0`).
* **Empirical Python Upstream Bug Verification**:
  ```python
  import textdistance
  textdistance.Cosine(qval=2)('0', '1')
  ```
  **Traceback**:
  ```text
  Traceback (most recent call last):
    File "<string>", line 1, in <module>
      import textdistance; print(textdistance.Cosine(qval=2)('0', '1'))
                                 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^
    File "textdistance/algorithms/token_based.py", line 217, in __call__
      return intersection / pow(prod, 1.0 / len(sequences))
             ~~~~~~~~~~~~~^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
  ZeroDivisionError: division by zero
  ```
* **Reasoning**:
  * In Python `textdistance`, `Cosine(qval=2)('0', '1')` raises `ZeroDivisionError: division by zero` because `prod` evaluates to 0 when inputs are shorter than $q$-gram length $q$.
  * Rust evaluates raw string identity (`req.s1 == req.s2`) first, returning `1.0` for identical inputs (e.g. `("", "")`), and returns `0.0` when `prod == 0.0` for non-identical inputs, ensuring 100% memory safety.

---

## 12. `Tanimoto` Distance / Similarity ($\log_2(\text{Jaccard})$) & `serde_json` Infinity Representation
* **Decision**: Implement Tanimoto formula $\log_2(\text{Jaccard}(s1, s2))$, returning `0.0` for raw string identity (`req.s1 == req.s2`) and `f64::NEG_INFINITY` for zero-Jaccard disjoint inputs. Handle `serde_json` `null` serialization of IEEE 754 infinity values in the Python fuzzing harness.
* **Empirical Python Upstream Behavior Verification**:
  ```python
  import textdistance
  t = textdistance.Tanimoto()
  print(t('', ''))           # 0.0
  print(t('cat', 'hat'))     # -1.0  (log2(0.5))
  print(t('abc', 'def'))     # -inf  (log2(0.0))

  ```
* **Reasoning**:
  * Tanimoto is defined as $\log_2(\text{Jaccard})$. When inputs are disjoint, Jaccard similarity is `0.0`, making $\log_2(0.0) = -\infty$.
  * Standard JSON specification does not define literal `Infinity` or `-Infinity` values. `serde_json` serializes `f64::NEG_INFINITY` as `null`. The Python differential fuzzing harness (`fuzz_driver.py`) explicitly maps `rust_val is None` to `math.isinf(py_val)` when validating `-inf` / `inf` outputs.

## 13. `Sorensen` Similarity (Dice Coefficient) & Total-Count Zero Protection
* **Decision**: Implement Sorensen-Dice formula $\frac{2 \times |A \cap B|}{|A| + |B|}$ using multiset/set counts, handling `total_count == 0` safely in Rust (`total_count == 0` $\rightarrow$ `similarity = 0.0`).
* **Empirical Python Upstream Bug Verification**:
  ```python
  import textdistance
  textdistance.Sorensen(qval=2)('0', '1')
  ```
  **Traceback**:
  ```text
  Traceback (most recent call last):
    File "<string>", line 1, in <module>
      import textdistance; print(textdistance.Sorensen(qval=2)('0', '1'))
                                 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^
    File "textdistance/algorithms/token_based.py", line 88, in __call__
      return 2.0 * intersection / count
             ~~~~~~~~~~~~~~~~~~~^~~~~~~
  ZeroDivisionError: division by zero
  ```
* **Reasoning**:
  * In Python `textdistance`, `Sorensen(qval=2)('0', '1')` raises `ZeroDivisionError: division by zero` because `count` (`count1 + count2`) evaluates to 0 when inputs are shorter than $q$-gram length $q$.
  * Rust evaluates raw string identity (`req.s1 == req.s2`) first, returning `1.0` for identical inputs (e.g. `("", "")`), and returns `0.0` when `total_count == 0` for non-identical inputs, ensuring 100% memory safety.

## 14. `Tversky` Index Parameterization ($\alpha, \beta, \text{bias}$) & Bias-Dependent Empty Token Evaluation
* **Decision**: Implement generalized Tversky index formula with configurable asymmetry weights $\alpha$ (default `1.0`), $\beta$ (default `1.0`), and additive `bias` (`Option<f64>`).
* **Empirical Python Behavior & Edge Case Verification**:
  * Default `Tversky(alpha=1, beta=1)` is identical to `Jaccard`.
  * `Tversky(alpha=0.5, beta=0.5)` is identical to `Sorensen` (Dice).
  * When `bias` is provided (e.g. `bias=1.0`), if short inputs produce empty token lists `[]` (len 0), Python evaluates $\frac{\text{inter} + \text{bias}}{\text{result} + \text{inter} + \text{bias}} = \frac{0 + 1.0}{0 + 0 + 1.0} = 1.0$.
  * Rust accurately models this dual branch (unbiased $0.0$ fallback vs biased ratio evaluation), achieving 100% differential parity.

## 15. `Bag` Distance Metric & Raw Input Length Normalization
* **Decision**: Implement Bag distance formula $\max(|A \setminus B|, |B \setminus A|)$ using multiset difference counts, deriving `maximum` as $\max(\text{len}(s1), \text{len}(s2))$ based on raw sequence lengths.
* **Empirical Python Behavior & Edge Case Verification**:
  * Inherits from `_Base` (distance metric). Python `Bag` does not accept `as_set` argument (only `qval`).
  * `Bag.maximum` returns $\max(\text{len}(s1), \text{len}(s2))$, regardless of $q$-gram window size.
  * For asymmetric inputs shorter than $q$-gram length $q$ (e.g. `('', '0')` with `qval=2`), $q$-gram token lists evaluate to `[]` (count 0), yielding `distance = 0.0`, `similarity = max_len = 1.0`.
  * Rust computes distance naturally from tokenized sequences while maintaining raw character length `maximum`, achieving 100% differential parity.

## 16. MRA (Match Rating Approach) — Phonetic Encoder & Standalone Design

### Context

`MRA` (Western Airlines Surname Match Rating Algorithm) is a phonetic string similarity algorithm whose core operating unit is not a token/char vector but a custom-encoded phonetic string. It inherits `_BaseSimilarity` in Python, meaning `similarity = __call__`, `distance = maximum - similarity`, `normalized_distance = distance / maximum` (0 when max=0), `normalized_similarity = 1 - normalized_distance`.

### Key Empirical Findings (verified against live Python library)

| Input pair | `__call__` | `.similarity()` | `.distance()` | `.maximum()` | `norm_sim` | `norm_dist` |
|---|---|---|---|---|---|---|
| `("", "")` | `0` | `0` | `0` | `0` | **`1`** | `0` |
| `("", "abc")` | `0` | `0` | `1` | `1` | `0.0` | `1.0` |
| `("cat", "cats")` | `2` | `2` | `1` | `3` | `0.667` | `0.333` |
| `("hello", "hello")` | `2` | `2` | `0` | `2` | `1.0` | `0.0` |
| `("a", "b")` | `0` | `0` | `1` | `1` | `0.0` | `1.0` |
| `("a", "bcdfg")` | `0` | `0` | `1` | `1` | `0.0` | `1.0` |

- `MRA.maximum()` computes `max(len(enc1), len(enc2))` where `enc1`, `enc2` are the **MRA-encoded strings**, not raw char counts.
- `("", "")`: Python `Base.normalized_distance` early-exits with `0` when `maximum == 0` → `normalized_similarity = 1 - 0 = 1`. Rust matches this by computing `normalized_distance = distance / maximum` with the `max==0 → 0.0` guard, then `normalized_similarity = 1.0 - 0.0 = 1.0`.
- `("", "abc")`: `__call__` returns `0` (early exit: `if not all(sequences)` fires because `""` is falsy). `maximum = max(0, 1) = 1`. `distance = 1 - 0 = 1`. `norm_sim = 0.0`.

### Encoding Algorithm (`_calc_mra`)

1. Uppercase the string.
2. Keep the first character; remove `A E I O U` from all subsequent characters.
3. Collapse consecutive duplicate characters (Unix `uniq`-style).
4. If the result is longer than 6 characters, keep first 3 + last 3.

```
"hello"    → "HELLO"   → "H"+"LL"   → dedup → "HL"       (len 2)
"world"    → "WORLD"   → "W"+"RLD"  → dedup → "WRLD"     (len 4)
"catherine"→ "CATHERINE"→"C"+"THRN"→ dedup → "CTHRN"    (len 5)
"kathryn"  → "KATHRYN" → "K"+"THRYN"→ dedup → "KTHRYN"  (len 6)
```

### Comparison Algorithm

For two strings (after encoding to `Vec<char>`):
1. If either **raw** input is empty → return `0`.
2. Compute `max_length = max(len(enc1), len(enc2))`. If `|len1 - len2| > 2` (i.e. `> count` where `count = 2`), return `0`.
3. Run **exactly `count = 2` iterations**:
   - Zip sequences positionally; keep pairs where chars are **not** equal.
   - Append the non-overlapping tail (beyond `min(len1, len2)`) of each sequence.
   - Update lengths.
4. Return `max_length - max(remaining_lengths)`.

### Design Decision: Standalone Struct (Not Generic Trait)

**Decision**: `Mra` struct exposes methods `compute()`, `similarity()`, `distance()`, `normalized_similarity()`, `normalized_distance()` operating on `&str` directly. It does NOT implement the generic `SimilarityMetric<T>` trait.

**Reasoning**: MRA's `maximum()` is defined over MRA-encoded lengths, not over the raw input slice length. Forcing it into `SimilarityMetric<char>` would require passing pre-encoded char vecs, which loses information about which chars are the "first character" (special rule: first char is always kept). The phonetic encoder is an integral part of the comparison logic, not a separable pre-processing step.

### Bug Caught During Implementation: NUL Char Dedup Sentinel

**Bug**: The dedup step in `calc_mra` initialized `prev = '\0'` (NUL). This caused any input beginning with a NUL character (`\x00`) to have that character silently dropped (since `'\x00' == '\0'`), making `calc_mra("\x00")` return `""` instead of `"\x00"`.

**Impact**: `maximum_score("", "\x00")` returned `0` instead of `1`, causing `distance("", "\x00")` to return `0.0` instead of `1.0`.

**Fix**: Changed `prev` from `'\0'` to `Option<char> = None`. The comparison `Some(c) != prev` correctly handles any char including NUL.

**Detection**: Hypothesis found the case `fuzz_mra(s1='', s2='\x00')` on the first fuzz run, before the fix was applied.

### IPC Handler

MRA receives raw `req.s1` and `req.s2` strings directly in `main.rs`. No `qval`, `as_set`, `alpha`, `beta`, or `bias` parameters apply.


## Log of Ported Algorithms & Decisions

| Algorithm | Base Type | Key Design Choices / Parity Rules | Status |
| :--- | :--- | :--- | :--- |
| **`Identity`** | `SimilarityMetric` | `s1 == s2` returns `1.0`, else `0.0`. Empty input `("", "")` returns `sim=1.0, dist=0.0`. | Verified (10k fuzz) |
| **`Length`** | `DistanceMetric` | Distance is `abs(len(s1) - len(s2))`. Maximum is `max(len(s1), len(s2))`. Empty input returns `dist=0.0, max=0.0`. | Verified (10k fuzz) |
| **`Prefix`** | `SimilarityMetric` | Exposes `prefix()` slice extractor and 2-track substring/numeric differential verification. | Verified (10k fuzz) |
| **`Postfix`** | `SimilarityMetric` | Exposes `postfix()` slice extractor and 2-track substring/numeric differential verification. | Verified (10k fuzz) |
| **`Matrix`** | `SimilarityMetric` | Custom substitution matrix via `[s1, s2, score]` triple list, symmetric lookup, identity fallback (`match_cost`), and mismatch fallback (`mismatch_cost`). | Verified (10k fuzz) |
| **`Jaccard`** | `SimilarityMetric` | Supports multiset and set intersections (`as_set`), $q$-gram tokenization (`qval`), raw identity quick_answer, and zero-union safety. | Verified (10k fuzz) |
| **`Overlap`** | `SimilarityMetric` | Supports multiset/set overlap coefficient $\frac{\|A \cap B\|}{\min(\|A\|, \|B\|)}$, $q$-gram tokenization (`qval`), raw identity quick_answer, and zero-count safety. | Verified (10k fuzz) |
| **`Cosine`** | `SimilarityMetric` | Supports multiset/set Cosine (Ochiai) coefficient $\frac{\|A \cap B\|}{\sqrt{\|A\| \times \|B\|}}$, $q$-gram tokenization (`qval`), raw identity quick_answer, and zero-product safety. | Verified (10k fuzz) |
| **`Tanimoto`** | `SimilarityMetric` | Computes $\log_2(\text{Jaccard})$, returning `0.0` for identical inputs and `f64::NEG_INFINITY` for disjoint inputs. | Verified (10k fuzz) |
| **`Sorensen`** | `SimilarityMetric` | Supports multiset/set Sorensen-Dice coefficient $\frac{2\|A \cap B\|}{\|A\| + \|B\|}$, $q$-gram tokenization (`qval`), raw identity quick_answer, and total-count safety. | Verified (10k fuzz) |
| **`Tversky`** | `SimilarityMetric` | Generalized asymmetric index with parameters $\alpha, \beta, \text{bias}$. Unifies Jaccard ($\alpha=1,\beta=1$) and Sorensen ($\alpha=0.5,\beta=0.5$). | Verified (10k fuzz) |
| **`Bag`** | `DistanceMetric` | Multiset difference distance $\max(\|A \setminus B\|, \|B \setminus A\|)$ with raw sequence length maximum normalization. | Verified (10k fuzz) |
| **`MRA`** | Standalone (phonetic) | Match Rating Approach operates on raw `&str` inputs via its own phonetic encoder; does not use generic `SimilarityMetric<T>` trait. `normalized_similarity("","") = 1.0`. | Verified (10k fuzz) |
| **`StrCmp95`** | Standalone / BaseSimilarity | Jaro-Winkler strcmp95 variant with phonetic/OCR character matrix (`sp_mx`), Winkler prefix scaling, optional `long_strings` tolerance adjustment, and Python whitespace trim. | Verified (10k fuzz) |

---

## 17. StrCmp95 (Jaro-Winkler strcmp95 Variant) — Phonetic/OCR Matrix & Whitespace Parity

### Context

`StrCmp95` implements the strcmp95 similarity algorithm (Winkler 1995). In Python reference `textdistance.algorithms.edit_based.StrCmp95`, it inherits `_BaseSimilarity`.
- `maximum` = 1.0
- `similarity` = `__call__`
- `distance` = `maximum - similarity` = `1.0 - similarity`
- `normalized_similarity` = `similarity`
- `normalized_distance` = `1.0 - similarity`

### Key Empirical Findings & Parity Rules

1. **Preprocessing & Quick Answer Parity**:
   - `s1` and `s2` are first trimmed of whitespace and converted to uppercase.
   - If cleaned `s1 == s2` (e.g. `("", "")`, `("a", "A")`, `("  ", "   ")`), returns `1.0`.
   - If either cleaned string is empty while the other is non-empty (`("", "a")`), returns `0.0`.
2. **ASCII C0 Control Code Whitespace Parity**:
   - Python's `str.strip()` strips all 29 Unicode whitespace characters, which includes ASCII C0 control characters 28 to 31 (`0x1C..=0x1F` / File, Group, Record, Unit separators).
   - Rust's `char::is_whitespace()` follows standard Unicode `White_Space` which omits `0x1C..=0x1F`.
   - **Fix**: Implemented `is_python_whitespace` helper in Rust (`c.is_whitespace() || matches!(c as u32, 0x1C..=0x1F)`) to match Python's `strip()` exact character set.
3. **Phonetic & OCR Substitution Matrix (`sp_mx`)**:
   - 36 character pairs (e.g., `'O'` and `'0'`, `'I'` and `'1'`, `'B'` and `'V'`) receive a partial match weight boost (`+3` weight / divided by 10) for unmatched characters within range `0 < ord(char) < 91`.
4. **Winkler Modification & `long_strings` Parameter**:
   - Prefix boost up to 4 characters applied if base weight `> 0.7`. Digits in prefix halt the prefix count.
   - Supports optional `long_strings` boolean parameter (`StrCmp95::with_config(long_strings)`), matching Python parameter signature.

---

### Step 14: `StrCmp95`

- **Whitespace Handling**: `StrCmp95` uses Python's `.strip()`, which removes not just spaces and tabs, but also all ASCII C0 control codes (`0x1C..=0x1F`). Our Rust implementation implements a custom `is_python_whitespace()` to match this behavior identically.
- **Early Exit**: `StrCmp95.__call__` explicitly calls `self.quick_answer()` AFTER calling `.strip().upper()`. This means `StrCmp95("  ", "   ")` strips down to empty strings, triggering the `quick_answer` empty string logic and returning `1.0`.
- **Default Parameters**: The Python default is `long_strings=False`.
- **Reference Values**: Our implementation perfectly matches Winkler's original test cases (e.g., `MARTHA` / `MARHTA` = `0.961111`, `shackleford` / `shackelford` = `0.981818`).

### Step 15: `Editex`

- **Type Mixing**: Python's `__call__` is annotated to return `float`, but computationally returns `int` (via integer matrix cells). Our Rust implementation correctly returns `usize` and converts to `f64` in the IPC harness to match the driver's generic expectations.
- **`maximum()` Calculation**: The maximum bound is computed using raw input character counts (`max(len(s1), len(s2)) * mismatch_cost`) *before* the `.upper()` transformation is applied, even if `.upper()` expands multi-byte characters.
- **Matrix Initialization Prepends**: The DP matrix initialization implicitly relies on prepending a space (`' '`) to strings before iterating (e.g., `s1 = ' ' + s1.upper()`). The space acts as an anchor for the `d_cost` and `r_cost` functions which evaluate `mismatch_cost` when interacting with spaces because they aren't in the initialized phonetic letter groups.
- **Methods**: Inheriting from `_Base` makes `Editex` behave as a distance metric fundamentally (`__call__` = `distance()`), with `similarity()` dynamically calculated as `maximum() - distance()`.

---
```
## MongeElkan — Scoped Out
MongeElkan was investigated and found to depend on an upstream bug in the reference Python implementation (`textdistance/algorithms/token_based.py`, line 267): `self.algorithm.maximum(sequences)` passes the pair of sequences as a single unstarred tuple argument instead of `*sequences`. This causes `maximum` to return an incorrect bound (e.g., `2` for empty inputs), leading to mismatched similarity results. Rather than replicate this bug, the project deliberately excludes MongeElkan from the Rust port scope.

* **Decision**: Do not implement MongeElkan in `textdistancerust`.
* **Reasoning**: The algorithm's behavior hinges on a documented upstream bug; preserving correctness requires either fixing the Python code (outside project scope) or omitting the algorithm. To keep the Rust implementation faithful and avoid reproducing known bugs, we scoped it out.
* **Action**: Removed module export and IPC match arm; no further code related to MongeElkan remains.

## 21. Gotoh Asymmetric Empty Input Crash Protection
* **Decision**: Handle asymmetric empty string inputs (one empty, one non-empty) by returning an explicit TextDistanceError::InvalidParameter.
* **Empirical Python Upstream Bug Verification**:
  `python
  import textdistance
  textdistance.Gotoh()('', 'a')
  ``r
  **Traceback**:
  `	ext
  Traceback (most recent call last):
    File "<string>", line 1, in <module> 
      import textdistance; print(textdistance.Gotoh().similarity('', 'a'))
    File "C:\Projects\Post_Mortem\textdistance\textdistance\algorithms\base.py", line 179, in similarity
      return self(*sequences)
    File "C:\Projects\Post_Mortem\textdistance\textdistance\algorithms\edit_based.py", line 619, in __call__
      p_mat[1, j] = -self.gap_open
  IndexError: index 1 is out of bounds for axis 0 with size 1
  ``r
* **Reasoning**:
  * In Python 	extdistance, Gotoh()('', 'a') raises IndexError because the DP matrix is initialized with xis 0 having size 1 (since s1 is empty), and then the inner loop attempts to write to index 1.
  * To maintain strict resilience without silently guessing  .0, the Rust implementation catches asymmetric empty inputs immediately and returns a descriptive error rather than evaluating the DP loops or assuming a mathematically dubious default score.

