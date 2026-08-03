# Text Distance Rust Conversion Summary

This document serves as a record of the Python to Rust algorithm conversion for the `Text_Distance_Rust` project. The primary goal was to port various text distance algorithms from the Python `textdistance` reference library to Rust while maintaining **100% logic and edge-case parity**.

## 🚀 Completed Algorithms

The following algorithms were successfully ported to Rust and validated against the original Python implementation:

### 1. Edit-Based Algorithms
- **Hamming**: Exact character mismatch counting.
- **Levenshtein**: Standard minimum edit distance (insertions, deletions, substitutions).
- **Jaro**: Window-based matching metric.
- **Jaro-Winkler**: Extension of Jaro with prefix scaling.
- **Needleman-Wunsch**: Global alignment with gap penalties.
- **Smith-Waterman**: Local alignment with gap penalties.
- **Gotoh**: Sequence alignment with affine gap penalties. *(Note: We fixed a minor index out-of-bounds bug in the original Python reference for empty strings to ensure it could be tested properly)*.
- **MLIPNS**: Bounded-mismatch iterative metric.

### 2. Sequence-Based Algorithms
- **LCSSeq (Longest Common Subsequence)**: Length of the longest non-contiguous common sequence computed using $O(N \cdot M)$ dynamic programming.
- **LCSStr (Longest Common Substring)**: Length of the longest contiguous common sequence.
- **Ratcliff-Obershelp (Gestalt Pattern Matching)**: A recursive similarity metric based on the longest common substring. 
  - *Implementation Detail:* The Rust version flawlessly reproduces the heuristic tie-breaking behavior of Python's `difflib.SequenceMatcher.find_longest_match()` exactly (which prefers matches that occur earliest in the first string, then earliest in the second string) to achieve 100% parity.

---

## 🛠️ Validation & Testing

Every single algorithm was rigorously verified using **Differential Fuzz Testing**:
- **Test Harness**: `fuzz_driver.py`
- **Fuzzing Engine**: Python `hypothesis` library.
- **Methodology**: 
  - The Python test harness generates random pairs of strings (ranging from empty strings to long Unicode strings).
  - It runs the original Python `textdistance` algorithm and simultaneously queries the compiled Rust binary (via a JSON over `stdio` IPC server).
  - The harness compares the outputs for `similarity`, `distance`, `normalized_similarity`, and `normalized_distance`.
- **Results**: Each algorithm passed **10,000 iterations** with **0 mismatches**.

---

## 🚫 Scoped Out Algorithms

As part of the initial planning phase, the following classes of algorithms were intentionally excluded from this conversion effort to maintain focus on the core edit/sequence logic:
- **Compression-Based Algorithms**: (e.g., LZ4, Zlib, BZ2, etc.)
- **Vector-Based / Token-Based Algorithms**: (e.g., Cosine, Jaccard, Tanimoto)
- **Monge-Elkan**: Scoped out as it was not required for the immediate conversion goals.

---

## 🏗️ Project Structure

- `textdistancerust/src/`
  - `lib.rs`: Exposes the public API and re-exports algorithms.
  - `main.rs`: The IPC server that handles fuzzing requests over standard I/O.
  - `traits.rs`: Defines the `SimilarityMetric` and `DistanceMetric` interfaces.
  - `[algorithm].rs`: The individual logic files for each algorithm.
- `fuzz-harness/fuzz_driver.py`: The Python test script used to orchestrate validation.
