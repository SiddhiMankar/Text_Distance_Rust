# Project Progress Log: `textdistancerust`

## Project Status Overview
- **Repository**: `textdistancerust/` (Standalone Rust package)
- **Active Track**: Person B (Tokenizer, Simple, Token-based, & Phonetic Metrics)
- **Current Step**: Step 6 (`Jaccard` Metric)
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
- Verified empty-input parity (`("", "")` returning `sim=match_cost (1.0), dist=0.0, max=1.0, norm_sim=1.0, norm_dist=0.0`).
- Added unit tests `test_matrix_default_same`, `test_matrix_default_different`, `test_matrix_empty`, `test_matrix_custom_map`.
- Integrated `"matrix"` string handler into `src/main.rs`.
- Fuzzed 10,000 iterations over persistent IPC with **0 mismatches in 6.19s**.

---

## Algorithm Checklist & Verification Status

| Step | Algorithm | Complexity | Expected Empty-Input Parity | Fuzz Iteration Target | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Step 0** | Tokenizer & Harness | Foundation | N/A | 10,000 | **DONE** |
| **Step 1** | `Identity` | Low | `sim=1.0, dist=0.0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 2** | `Length` | Low | `dist=0.0, max=0, norm_dist=0.0` | 10,000 | **DONE** |
| **Step 3** | `Prefix` | Low | `sim=0, max=0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 4** | `Postfix` | Low | `sim=0, max=0, norm_sim=1.0` | 10,000 | **DONE** |
| **Step 5** | `Matrix` | Low-Med | `sim=match_cost (1.0), norm_sim=1.0` | 10,000 | *IN PROGRESS* |
| **Step 6** | `Jaccard` | Med | `sim=1.0, norm_sim=1.0` (via quick_answer) | 10,000 | Pending |   
| **Step 7** | `Overlap` | Med | `sim=1.0, norm_sim=1.0` | 10,000 | Pending |
| **Step 8** | `Cosine` | Med | `sim=1.0, norm_sim=1.0` | 10,000 | Pending |
| **Step 9** | `Tanimoto` | Med | `sim=0.0` for `("", "")`; `-inf` for disjoint | 10,000 | Pending |
| **Step 10** | `Sorensen` | Med | `sim=1.0, norm_sim=1.0` | 10,000 | Pending |
| **Step 11** | `Tversky` | Med-High | `sim=1.0, norm_sim=1.0` | 10,000 | Pending |
| **Step 12** | `Bag` | Med | `dist=0.0, norm_dist=0.0` | 10,000 | Pending |
| **Step 13** | `MRA` | Med-High | `sim=0, max=0, norm_sim=1.0` | 10,000 | Pending |
| **Step 14** | `StrCmp95` | High | `sim=1.0, norm_sim=1.0` | 10,000 | Pending |
| **Step 15** | `Editex` | High | `dist=0.0, norm_dist=0.0` | 10,000 | Pending |
