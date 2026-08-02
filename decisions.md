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

---

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
