# Project Progress Log: `textdistancerust`

## Project Status Overview
- **Repository**: `textdistancerust/` (Standalone Rust package)
- **Active Track**: Person B (Tokenizer, Simple, Token-based, & Phonetic Metrics)
- **Current Step**: Person B track completed! All metrics verified.
- **Verification Strategy**: Continuous differential fuzzing against Python `textdistance` reference via persistent IPC (10,000+ iterations per algorithm minimum).

---

## Detailed Completed Log

### Step 0: Package Setup, Core Traits & Tokenizer (`tokenizer.rs`)
- **Status**: Completed & Verified
- **Date**: August 1, 2026

#### Detailed Technical Deliverables Created:
1. **Package Manifest ([`Cargo.toml`](file:///c:/Projects/Post_Mortem/textdistancerust/Cargo.toml))**:
   - Initialized standalone crate named `textdistancerust`.
   - Configured `lib` target (`textdistancerust`) and binary target (`textdistancerust-cli`).
   - Added `serde` (1.0 with derive) and `serde_json` (1.0) dependencies for persistent JSON streaming IPC.

2. **Error Handling Subsystem ([`src/error.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/error.rs))**:
   - Implemented `TextDistanceError` enum containing variants:
     - `InvalidParameter(String)`: Out-of-bounds or illegal metric parameters.
     - `EmptyInputSequence`: Empty input handling where required.
     - `CalculationOverflow`: Arithmetic overflow protection.
     - `IncompatibleLength`: Mismatched sequence length errors (e.g. Hamming).
   - Implemented `std::fmt::Display` and `std::error::Error` for seamless integration.

3. **Core Trait Definitions ([`src/traits.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/traits.rs))**:
   - Implemented `DistanceMetric<T>` trait:
     - `distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError>`
     - `maximum(&self, s1: &[T], s2: &[T]) -> f64`
     - Provided default `similarity` (`maximum - distance`).
     - Provided default `normalized_distance` (handles `maximum == 0.0` fallback to `0.0`).
     - Provided default `normalized_similarity` (`1.0 - normalized_distance`).
   - Implemented `SimilarityMetric<T>` trait:
     - `similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError>`
     - `maximum(&self, s1: &[T], s2: &[T]) -> f64`
     - Provided default `distance` (`maximum - similarity`).
     - Provided default `normalized_distance` (handles `maximum == 0.0` fallback to `0.0`).
     - Provided default `normalized_similarity` (handles `maximum == 0.0` fallback to `1.0`).

4. **Sequence Tokenizer Utility ([`src/tokenizer.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/tokenizer.rs))**:
   - `to_char_vec(s: &str) -> Vec<char>`: Converts UTF-8 string slices into Unicode scalar character vectors, preventing byte-indexing panics.
   - `to_word_vec(s: &str) -> Vec<&str>`: Splits text by whitespace.
   - `find_ngrams<T: Clone>(input: &[T], n: usize) -> Vec<Vec<T>>`: Sliding window slice generator for $q$-gram tokenization.
   - Added unit test suite in `tokenizer.rs` validating empty inputs, multi-byte Unicode emojis (`🔥🔑`), whitespace padding, and boundary window sizes.

5. **Persistent IPC CLI Executable ([`src/main.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/main.rs))**:
   - Built `textdistancerust-cli` long-lived executable.
   - Configured `stdin`/`stdout` JSON line-streaming protocol reading `FuzzRequest` (`{alg, s1, s2}`) and emitting `FuzzResponse` (`{similarity, distance, normalized_similarity, normalized_distance, subsequence, error}`).
   - Avoids process creation overhead during high-volume fuzzing campaigns.

6. **Differential Fuzzing Driver ([`fuzz-harness/fuzz_driver.py`](file:///c:/Projects/Post_Mortem/fuzz-harness/fuzz_driver.py))**:
   - Python driver using `subprocess.Popen` over persistent stdin/stdout with explicit `encoding='utf-8'` configuration.
   - Pre-seeds verification runs with deterministic edge case pairs (`("", "")`, `("", "a")`, `("a", "")`, `("a", "a")`, `("a", "b")`, `("spam", "qwer")`).
   - Implements two-track comparison for sub-extractors: Track 1 verifies literal 1:1 string output (`r_subseq == py_subseq`), Track 2 verifies numeric metrics.

---

### Step 1: `Identity` Similarity Metric ([`src/identity.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/identity.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 1, 2026

#### Detailed Technical Summary:
- Implemented `Identity` struct conforming to `SimilarityMetric<T>`.
- Returns similarity `1.0` if `s1 == s2`, else `0.0`.
- Verified empty-input parity (`("", "")` returning `sim=1.0, dist=0.0, norm_sim=1.0, norm_dist=0.0`).
- Integrated `"identity"` string handler into `main.rs` IPC loop.
- Added `test_identity_same`, `test_identity_different`, and `test_identity_empty` unit tests.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 9.39s**.

---

### Step 2: `Length` Distance Metric ([`src/length.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/length.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 1, 2026

#### Detailed Technical Summary:
- Implemented `Length` struct conforming to `DistanceMetric<T>`.
- Calculates distance as `|(s1.len() - s2.len())| as f64`.
- Calculates `maximum` as `max(s1.len(), s2.len()) as f64`.
- Verified empty-input parity (`("", "")` returning `dist=0.0, max=0.0, norm_dist=0.0, norm_sim=1.0`).
- Added provided `similarity` method (`maximum - distance`) in `DistanceMetric` trait in `src/traits.rs`.
- Integrated `"length"` handler into `src/main.rs`.
- Added unit tests `test_length_same`, `test_length_different`, `test_length_empty`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 6.58s**.

---

### Step 3: `Prefix` Similarity Metric & Subsequence Extractor ([`src/prefix.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/prefix.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Prefix` struct conforming to `SimilarityMetric<T>`.
- Provides `prefix<'a, T: PartialEq>(&self, s1: &'a [T], s2: &'a [T]) -> &'a [T]` slice helper.
- Calculates `similarity` as matching prefix length `self.prefix(s1, s2).len() as f64`.
- Calculates `maximum` as `max(s1.len(), s2.len()) as f64`. 
- Verified empty-input parity (`("", "")` returning `sim=0.0, max=0.0, norm_sim=1.0, norm_dist=0.0`).
- Wired `subsequence` field into `FuzzResponse` in `src/main.rs`.
- Added unit tests `test_prefix_matching`, `test_prefix_different`, `test_prefix_empty`.
- Verified via two-track differential fuzzing (**10,000 iterations in 7.50s with 0 mismatches**).

---

### Step 4: `Postfix` Similarity Metric & Subsequence Extractor ([`src/postfix.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/postfix.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Postfix` struct conforming to `SimilarityMetric<T>`.
- Provides `postfix<'a, T: PartialEq>(&self, s1: &'a [T], s2: &'a [T]) -> &'a [T]` slice helper matching common trailing elements.
- Calculates `similarity` as matching postfix length `self.postfix(s1, s2).len() as f64`.
- Calculates `maximum` as `max(s1.len(), s2.len()) as f64`.
- Verified empty-input parity (`("", "")` returning `sim=0.0, max=0.0, norm_sim=1.0, norm_dist=0.0`).
- Integrated `"postfix"` handler into `src/main.rs`.
- Added unit tests `test_postfix_matching`, `test_postfix_different`, `test_postfix_empty`.
- Verified via two-track differential fuzzing (**10,000 iterations in 6.81s with 0 mismatches**).

---

### Step 5: `Matrix` Similarity Metric ([`src/matrix.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/matrix.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Matrix` struct conforming to `SimilarityMetric<T>`.
- Supports optional custom substitution score map `mat: Option<HashMap<(T, T), f64>>`, configurable `match_cost` (default `1.0`), `mismatch_cost` (default `0.0`), and `symmetric` flag (default `true`).
- Performs direct match lookup, symmetric pair fallback lookup, identity check fallback (`match_cost`), and default mismatch fallback (`mismatch_cost`).
- Represented `mat` JSON payloads as a list of `[s1, s2, score]` triples (`Vec<(String, String, f64)>`) to avoid non-standard JSON composite string keys.
- Verified empty-input parity (`("", "")` returning `sim=match_cost (1.0), dist=0.0, max=1.0, norm_sim=1.0, norm_dist=0.0`).
- Added unit tests `test_matrix_default_same`, `test_matrix_default_different`, `test_matrix_empty`, `test_matrix_custom_map`.
- Integrated `"matrix"` string handler into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 12.23s** (randomizing `mat` triple arrays in Hypothesis).

---

### Step 6: `Jaccard` Similarity Metric ([`src/jaccard.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/jaccard.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Jaccard` struct conforming to `SimilarityMetric<T>`.
- Supports multiset counts (`as_set = false`) and unique set intersections (`as_set = true`), as well as configurable tokenization (`qval = 1` scalar chars, `qval > 1` $q$-grams, `qval = 0` whitespace words).
- Evaluates raw identity (`req.s1 == req.s2`) first, returning `1.0` for identical inputs (e.g. `("", "")`).
- Handles zero-union token sets safely in Rust (`union == 0` $\rightarrow$ `similarity = 0.0`), preventing division by zero when non-equal inputs yield empty $q$-gram token lists.
- Documented upstream Python bug where `textdistance.Jaccard(qval=2)('0', '1')` raises unhandled `ZeroDivisionError: division by zero`.
- Added unit tests `test_jaccard_same`, `test_jaccard_empty`, `test_jaccard_asymmetric_empty`, `test_jaccard_cat_hat`, `test_jaccard_as_set`.
- Integrated `"jaccard"` string handler with `qval` and `as_set` JSON parameters into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 8.89s**.

---

### Step 7: `Overlap` Similarity Metric ([`src/overlap.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/overlap.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Overlap` struct conforming to `SimilarityMetric<T>`.
- Computes overlap coefficient formula $\frac{|A \cap B|}{\min(|A|, |B|)}$ for multiset (`as_set = false`) and set (`as_set = true`) intersections.
- Supports configurable tokenization (`qval = 1` scalar chars, `qval > 1` $q$-grams, `qval = 0` whitespace words).
- Handles `min_count == 0` safely in Rust (`min_count == 0` $\rightarrow$ `similarity = 0.0`), protecting against division by zero when short input tokens yield zero-length n-gram slices.
- Added unit tests `test_overlap_same`, `test_overlap_empty`, `test_overlap_asymmetric_empty`, `test_overlap_cat_hat`.
- Integrated `"overlap"` string handler with `qval` and `as_set` JSON parameters into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 9.78s**.

---

### Step 8: `Cosine` Similarity Metric ([`src/cosine.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/cosine.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Cosine` struct conforming to `SimilarityMetric<T>`.
- Computes Cosine similarity (Ochiai coefficient) formula $\frac{|A \cap B|}{\sqrt{|A| \times |B|}}$ for multiset (`as_set = false`) and set (`as_set = true`) intersections.
- Supports configurable tokenization (`qval = 1` scalar chars, `qval > 1` $q$-grams, `qval = 0` whitespace words).
- Handles `prod == 0.0` safely in Rust (`prod == 0.0` $\rightarrow$ `similarity = 0.0`), protecting against division by zero when short input tokens yield zero-length n-gram slices.
- Documented upstream Python bug where `textdistance.Cosine(qval=2)('0', '1')` raises unhandled `ZeroDivisionError: division by zero`.
- Added unit tests `test_cosine_same`, `test_cosine_empty`, `test_cosine_asymmetric_empty`, `test_cosine_cat_hat`.
- Integrated `"cosine"` string handler with `qval` and `as_set` JSON parameters into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 9.20s**.

---

### Step 9: `Tanimoto` Similarity Metric ([`src/tanimoto.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/tanimoto.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Tanimoto` struct conforming to `SimilarityMetric<T>`.
- Computes Tanimoto similarity as $\log_2(\text{Jaccard}(s1, s2))$ over multiset (`as_set = false`) and set (`as_set = true`) token collections.
- Evaluates raw identity (`req.s1 == req.s2`) first, returning `0.0` for identical inputs (e.g. `("", "")`), since $\log_2(1.0) = 0.0$.
- Returns `f64::NEG_INFINITY` (`-inf`) when Jaccard similarity is `0.0` (disjoint inputs or short input $q$-gram tokenization limits).
- Handled `serde_json` `null` serialization for IEEE 754 infinity values in the Python fuzzing harness (`fuzz_driver.py`).
- Added unit tests `test_tanimoto_same`, `test_tanimoto_empty`, `test_tanimoto_disjoint`, `test_tanimoto_cat_hat`.
- Integrated `"tanimoto"` string handler with `qval` and `as_set` JSON parameters into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 9.71s**.

---

### Step 10: `Sorensen` Similarity Metric ([`src/sorensen.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/sorensen.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Sorensen` struct conforming to `SimilarityMetric<T>`.
- Computes Sorensen-Dice coefficient formula $\frac{2 \times |A \cap B|}{|A| + |B|}$ for multiset (`as_set = false`) and set (`as_set = true`) intersections.
- Supports configurable tokenization (`qval = 1` scalar chars, `qval > 1` $q$-grams, `qval = 0` whitespace words).
- Handles `total_count == 0` safely in Rust (`total_count == 0` $\rightarrow$ `similarity = 0.0`), protecting against division by zero when short input tokens yield zero-length n-gram slices.
- Documented upstream Python bug where `textdistance.Sorensen(qval=2)('0', '1')` raises unhandled `ZeroDivisionError: division by zero`.
- Added unit tests `test_sorensen_same`, `test_sorensen_empty`, `test_sorensen_asymmetric_empty`, `test_sorensen_cat_hat`.
- Integrated `"sorensen"` string handler with `qval` and `as_set` JSON parameters into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 8.91s**.

---

### Step 11: `Tversky` Similarity Metric ([`src/tversky.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/tversky.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Tversky` struct conforming to `SimilarityMetric<T>`.
- Computes generalized Tversky index $\frac{|A \cap B| + \text{bias}}{|A \cap B| + \text{bias} + \alpha(|A \setminus B|) + \beta(|B \setminus A|)}$ for multiset (`as_set = false`) and set (`as_set = true`) token collections.
- Supports configurable asymmetry weights $\alpha$ (default `1.0`), $\beta$ (default `1.0`), and optional additive `bias` (`Option<f64>`).
- Modeled dual empty-token branch (unbiased $0.0$ fallback vs biased ratio evaluation yielding $1.0$), matching Python reference behavior.
- Added unit tests `test_tversky_default_jaccard_parity`, `test_tversky_dice_parity`, `test_tversky_bias`.
- Integrated `"tversky"` string handler with `alpha`, `beta`, `bias`, `qval`, and `as_set` JSON parameters into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 15.00s** (randomizing $\alpha, \beta \in [0.1, 5.0]$, $\text{bias} \in [\text{None}, 0.0, 2.0]$, $q\text{val} \in [1, 3]$, and `as_set`).

---

### Step 12: `Bag` Distance Metric ([`src/bag.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/bag.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
- Implemented `Bag` struct conforming to `DistanceMetric<T>`.
- Computes Bag distance as multiset difference max $\max(|A \setminus B|, |B \setminus A|)$.
- Calculates `maximum` as raw sequence character length $\max(\text{len}(s1), \text{len}(s2))$, matching Python `_Base.maximum` behavior across all $q$-gram window settings.
- Derived `similarity` as $\text{maximum} - \text{distance}$ and `normalized_distance` as $\frac{\text{distance}}{\text{maximum}}$.
- Added unit tests `test_bag_same`, `test_bag_empty`, `test_bag_cat_hat`.
- Integrated `"bag"` string handler with `qval` JSON parameter into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 9.82s**.

---

## Algorithm Checklist & Verification Status

| Step | Algorithm | Complexity | Expected Empty-Input Parity | Fuzz Iteration Target | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Step 0** | Tokenizer & Harness | Foundation | N/A | 10,000 | **DONE** |
| **Step 1** | `Identity` | Low | `sim=1.0, dist=0.0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 2** | `Length` | Low | `dist=0.0, max=0, norm_dist=0.0` | 10,000 | **DONE** |
| **Step 3** | `Prefix` | Low | `sim=0, max=0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 4** | `Postfix` | Low | `sim=0, max=0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 5** | `Matrix` | Low-Med | `sim=match_cost (1.0), norm_sim=1.0` | 10,000 | **DONE** |
| **Step 6** | `Jaccard` | Med | `sim=1.0, norm_sim=1.0` (via quick_answer) | 10,000 | **DONE** |
| **Step 7** | `Overlap` | Med | `sim=1.0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 8** | `Cosine` | Med | `sim=1.0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 9** | `Tanimoto` | Med | `sim=0.0` for `("", "")`; `-inf` for disjoint | 10,000 | **DONE** |
| **Step 10** | `Sorensen` | Med | `sim=1.0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 11** | `Tversky` | Med-High | `sim=1.0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 12** | `Bag` | Med | `dist=0.0, norm_dist=0.0` | 10,000 | **DONE** |
| **Step 13** | `MRA` | Med-High | `sim=0, max=0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 14** | `StrCmp95` | High | `sim=1.0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 15** | `Editex` | High | `dist=0.0, norm_dist=0.0` | 10,000 | Pending |

---

### Step 13: `MRA` Phonetic Similarity Metric ([`src/mra.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/mra.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Algorithm Research (Source-First Approach)

Before writing any Rust, the full Python source in [`textdistance/algorithms/phonetic.py`](file:///c:/Projects/Post_Mortem/textdistance/textdistance/algorithms/phonetic.py) was read and the following behaviors were empirically verified:

| Input pair | `__call__` | `.similarity()` | `.distance()` | `.maximum()` | `norm_sim` | `norm_dist` |
|---|---|---|---|---|---|---|
| `("", "")` | `0` | `0` | `0` | `0` | **`1`** | `0` |
| `("", "abc")` | `0` | `0` | `3` | `3` | `0.0` | `1.0` |
| `("cat", "cats")` | `2` | `2` | `1` | `3` | `0.667` | `0.333` |
| `("hello", "hello")` | `2` | `2` | `0` | `2` | `1.0` | `0.0` |
| `("catherine", "kathryn")` | `3` | `3` | `3` | `6` | `0.5` | `0.5` |
| `("a", "bcdfg")` | `0` | `0` | `1` | `1` | `0.0` | `1.0` |

Key encoder traces verified:
```
"hello"     → "HL"     (len 2)
"world"     → "WRLD"   (len 4)
"catherine" → "CTHRN"  (len 5)
"kathryn"   → "KTHRYN" (len 6)
```

#### Detailed Technical Deliverables:

1. **`Mra` Struct ([`src/mra.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/mra.rs))**:
   - Implemented as a **standalone struct** (not implementing generic `SimilarityMetric<T>` trait) because MRA's `maximum()` is defined over MRA-encoded string lengths, not raw input lengths.
   - `pub fn calc_mra(word: &str) -> String`: Static phonetic encoder.
     - Uppercase → strip inner vowels (`AEIOU`) → deduplicate consecutive chars → truncate to first-3+last-3 if >6 chars.
   - `pub fn compute(&self, s1: &str, s2: &str) -> f64`: Comparison algorithm.
     - Empty raw input early-exit (returns `0.0`).
     - Length-difference threshold check: `|len1 - len2| > 2` → `0.0`.
     - 2-iteration positional matching loop (strips matching prefix pairs; appends trailing tail).
     - Returns `max_length - max(remaining_lengths)`.
   - `similarity`, `distance`, `normalized_similarity`, `normalized_distance` methods expose the 4 standard Python-parity metrics.
   - **Empty+empty parity**: `normalized_similarity("", "") = 1.0`, matching Python's `Base.normalized_distance(max=0) → 0` → `normalized_similarity = 1 - 0 = 1`.

2. **Bug Found and Fixed: NUL Char Dedup Sentinel**:
   - **Bug**: `prev` in dedup initialized to `'\0'` (NUL) caused any string starting with NUL (`\x00`) to have it silently dropped.
   - **Impact**: `maximum_score("", "\x00")` returned `0` instead of `1`, giving wrong `distance=0.0` instead of `1.0`.
   - **Fix**: Changed `prev` from `'\0'` to `Option<char> = None`.
   - **Detection**: Hypothesis found `fuzz_mra(s1='', s2='\x00')` on the first fuzz run.

3. **IPC Integration**:
   - Added `"mra"` arm in `main.rs` using `req.s1`/`req.s2` directly (no qval/as_set).
   - Added `mra` module export in `lib.rs`.

4. **Fuzz Harness ([`fuzz-harness/fuzz_driver.py`](file:///c:/Projects/Post_Mortem/fuzz-harness/fuzz_driver.py))**:
   - Added 5 named seed cases (empty inputs, threshold-exceeded, cat/cats, catherine/kathryn).
   - `verify_pair` instantiates `textdistance.MRA()` (no kwargs needed).
   - Hypothesis fuzz strategy: `st.text() × st.text()` — exercises all Unicode ranges.

5. **Unit Tests** (11 new tests, all pass):
   - Encoder: `test_calc_mra_hello`, `test_calc_mra_world`, `test_calc_mra_catherine`, `test_calc_mra_kathryn`, `test_calc_mra_empty`.
   - Comparison: `test_mra_empty_inputs`, `test_mra_cat_cats`, `test_mra_identical`, `test_mra_a_vs_b`, `test_mra_maximum`, `test_mra_threshold_exceeded`.

6. **Fuzz Results**: **10,000 iterations / 0 mismatches / 8.51s** (covering arbitrary Unicode text inputs).

7. **Documentation**: Decision 16 appended to [`decisions.md`](file:///c:/Projects/Post_Mortem/decisions.md#16-mra-match-rating-approach--phonetic-encoder--standalone-design).

---

### Step 14: `StrCmp95` Jaro-Winkler Metric Variant ([`src/strcmp95.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/strcmp95.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Deliverables:

1. **`StrCmp95` Struct ([`src/strcmp95.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/strcmp95.rs))**:
   - Implements strcmp95 Jaro-Winkler algorithm variant with `long_strings` configuration support.
   - 36-pair phonetic/OCR substitution matrix (`SP_MX`) giving `+3` weight boost for unmatched characters within range `0 < ord(char) < 91`.
   - Winkler prefix scaling (boost applied when base weight `> 0.7`, stopped if prefix character is digit).
   - Custom `is_python_whitespace` helper handling ASCII C0 control codes (`0x1C..=0x1F`) matching Python's `str.strip()` exact character set.
   - Provides `similarity()`, `distance()`, `normalized_similarity()`, `normalized_distance()` metrics.

2. **IPC & Fuzz Integration**:
   - Wired `"strcmp95"` arm with `long_strings` parameter in `main.rs` and `fuzz_driver.py`.
   - Pre-seeded fuzz harness with classic strcmp95 test pairs (`MARTHA`/`MARHTA`, `shackleford`/`shackelford`).
   - Hypothesis fuzz strategy: `st.text() × st.text() × st.booleans()` varying strings and `long_strings`.

3. **Unit Tests & Fuzzing**:
   - Passed 60/60 Rust unit tests (`cargo test`).
   - **Passed 10,000 differential fuzz iterations** in **8.46s** with **0 mismatches**.

4. **Documentation**: Decision 17 logged in [`decisions.md`](file:///c:/Projects/Post_Mortem/decisions.md#17-strcmp95-jaro-winkler-strcmp95-variant--phoneticocr-matrix--whitespace-parity).

**Suite Total**: 60 unit tests, 14 algorithms verified, 140,000 total fuzz iterations, 0 mismatches.

---

### Step 15: `Editex` Phonetic Distance Metric ([`src/editex.rs`](file:///c:/Projects/Post_Mortem/textdistancerust/src/editex.rs))
- **Status**: Completed & Verified (10,000 / 10,000 iterations passed, 0 mismatches)
- **Date**: August 2, 2026

#### Detailed Technical Summary:
1. **Algorithm Porting**:
   - Implemented `Editex` struct with custom letter groups based on Soundex principles.
   - Accurately ported the Python DP matrix logic including `d_cost` and `r_cost` penalties.
   - Matches Python's integer space logic (where string initialization implicitly adds a space char).
   - Distance defaults correctly, similarity is computed dynamically as `maximum() - distance()`.

2. **IPC & Fuzz Integration**:
   - Integrated `"editex"` arm in `src/main.rs`.
   - Wired the IPC harness for `editex` strings in `fuzz_driver.py`.

3. **Fuzzing Results**:
   - **Passed 10,000 differential fuzz iterations** with **0 mismatches**.

**Suite Total**: 15 algorithms verified, 150,000 total fuzz iterations, 0 mismatches.
