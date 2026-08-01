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

---

## Log of Ported Algorithms & Decisions

| Algorithm | Base Type | Key Design Choices / Parity Rules | Status |
| :--- | :--- | :--- | :--- |
| **`Identity`** | `SimilarityMetric` | `s1 == s2` returns `1.0`, else `0.0`. Empty input `("", "")` returns `sim=1.0, dist=0.0`. | Verified (10k fuzz) |
| **`Length`** | `DistanceMetric` | Distance is `abs(len(s1) - len(s2))`. Maximum is `max(len(s1), len(s2))`. Empty input returns `dist=0.0, max=0.0`. | Verified (10k fuzz) |
| **`Prefix`** | `SimilarityMetric` | Exposes `prefix()` slice extractor and 2-track substring/numeric differential verification. | Verified (10k fuzz) |
| **`Postfix`** | `SimilarityMetric` | Exposes `postfix()` slice extractor and 2-track substring/numeric differential verification. | Verified (10k fuzz) |
| **`Matrix`** | `SimilarityMetric` | Custom substitution matrix via `[s1, s2, score]` triple list, symmetric lookup, identity fallback (`match_cost`), and mismatch fallback (`mismatch_cost`). | Verified (10k fuzz) |
