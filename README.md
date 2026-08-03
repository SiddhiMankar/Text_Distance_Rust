# `textdistancerust`: High-Performance Rust Port of Python `textdistance`

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Unsafe Code](https://img.shields.io/badge/unsafe-forbid-blue.svg)]()
[![Fuzz Verification](https://img.shields.io/badge/fuzzing-10000%2B%20iters%2Falg-success.svg)]()
[![Docker Ready](https://img.shields.io/badge/docker-ready-blue.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

`textdistancerust` is a standalone, safe Rust reimplementation of Python's popular sequence distance library [`life4/textdistance`](https://github.com/life4/textdistance).

It delivers **100% behavioral equivalence** (down to $\le 10^{-9}$ floating-point precision) against the Python reference implementation across **30 algorithms**, validated with differential property-based fuzz testing (10,000+ iterations per algorithm, 0 mismatches) and a 89-test unit/integration test suite.

---

## Table of Contents
- [Key Features & Architecture](#key-features--architecture)
- [Interactive CUI Toolkit (`tdcli`)](#interactive-cui-toolkit-tdcli)
- [Summary of Algorithms](#summary-of-algorithms)
- [Local Building & Testing](#local-building--testing)
- [Differential Fuzz Harness](#differential-fuzz-harness)
- [Docker Setup](#docker-setup)
- [Repository Structure](#repository-structure)
- [License](#license)

---

## Key Features & Architecture

1. **Zero Python Runtime Dependency**:
   - Compiles to a clean Rust library ([`libtextdistancerust.rlib`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/lib.rs)) and standalone binaries ([`tdcli`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/bin/tdcli.rs) & [`textdistancerust-cli`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/main.rs)).
2. **Strict Zero `unsafe` Policy**:
   - The crate enforces `#![forbid(unsafe_code)]` at compile time.
3. **Generic Sequence Support (`&[T]`)**:
   - Operates over generic slice sequences (`&[T]`) rather than plain byte strings, avoiding UTF-8 boundary panics on multi-byte Unicode characters (CJK, Emojis) and supporting token vectors (`Vec<&str>`).
4. **Dual Trait Abstraction**:
   - [`DistanceMetric<T>`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/traits.rs#L3-L22): Core interface for edit/distance metrics (`distance()`, `maximum()`, derived `similarity()`, `normalized_distance()`, `normalized_similarity()`).
   - [`SimilarityMetric<T>`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/traits.rs#L24-L47): Core interface for token/sequence metrics (`similarity()`, `maximum()`, derived `distance()`, `normalized_similarity()`, `normalized_distance()`).
5. **Edge-Case Parity**:
   - Implements exact Python edge behavior for zero-maximum, empty-input, and zero-count edge cases without panicking.

---

## Interactive CUI Toolkit (`tdcli`)

The repository includes a zero-dependency interactive command-line user interface (**CUI**) featuring ANSI colors, emoji category tags, and Unicode box-drawing result tables.

```
Usage: cargo run --bin tdcli <COMMAND> [ARGS...]
```

### Commands

| Command | Description | Example |
| :--- | :--- | :--- |
| `list` | Lists all 30 algorithms grouped by category | `cargo run --bin tdcli list` |
| `compare` | Compares two strings using specified algorithm(s) | `cargo run --bin tdcli compare --alg levenshtein,jaccard "kitten" "sitting"` |
| `all` | Runs all 30 algorithms and displays a ranked comparison table | `cargo run --bin tdcli all "MARTHA" "MARHTA"` |
| `bench` | Runs in-process microsecond latency benchmarks | `cargo run --bin tdcli bench` |
| `interactive` | Launches an interactive REPL session | `cargo run --bin tdcli interactive` |

---

## Summary of Algorithms

| Category | Algorithm | Rust Module | Description | Trait | Fuzz Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Simple** | `Identity` | [`identity.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/identity.rs) | Exact sequence identity | `SimilarityMetric` | PASSED (10k) |
| | `Length` | [`length.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/length.rs) | Length difference metric | `DistanceMetric` | PASSED (10k) |
| | `Prefix` | [`prefix.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/prefix.rs) | Common prefix fraction | `SimilarityMetric` | PASSED (10k) |
| | `Postfix` | [`postfix.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/postfix.rs) | Common postfix fraction | `SimilarityMetric` | PASSED (10k) |
| **Matrix** | `Matrix` | [`matrix.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/matrix.rs) | Custom score matrix matching | `SimilarityMetric` | PASSED (10k) |
| **Edit** | `Hamming` | [`hamming.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/hamming.rs) | Positional mismatch count | `DistanceMetric` | PASSED (10k) |
| | `Levenshtein` | [`levenshtein.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/levenshtein.rs) | Minimum edit distance (ins/del/sub) | `DistanceMetric` | PASSED (10k) |
| | `DamerauLevenshtein` | [`damerau_levenshtein.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/damerau_levenshtein.rs) | OSA edit distance with transpositions | Both traits | PASSED (10k) |
| | `Jaro` | [`jaro.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/jaro.rs) | Jaro character-window metric | `SimilarityMetric` | PASSED (10k) |
| | `JaroWinkler` | [`jaro_winkler.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/jaro_winkler.rs) | Jaro with prefix scaling | `SimilarityMetric` | PASSED (10k) |
| | `StrCmp95` | [`strcmp95.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/strcmp95.rs) | Jaro-Winkler strcmp95 phonetic variant | `SimilarityMetric` | PASSED (10k) |
| | `Mlipns` | [`mlipns.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/mlipns.rs) | Bounded mismatch iterative metric | `SimilarityMetric` | PASSED (10k) |
| **Alignment** | `NeedlemanWunsch` | [`needleman_wunsch.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/needleman_wunsch.rs) | Global sequence alignment | `SimilarityMetric` | PASSED (10k) |
| | `SmithWaterman` | [`smith_waterman.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/smith_waterman.rs) | Local sequence alignment | `SimilarityMetric` | PASSED (10k) |
| | `Gotoh` | [`gotoh.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/gotoh.rs) | Alignment with affine gap penalties | `SimilarityMetric` | PASSED (10k) |
| **Sequence** | `LcsSeq` | [`lcsseq.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/lcsseq.rs) | Longest common subsequence | `SimilarityMetric` | PASSED (10k) |
| | `LcsStr` | [`lcsstr.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/lcsstr.rs) | Longest common substring | `SimilarityMetric` | PASSED (10k) |
| | `RatcliffObershelp` | [`ratcliff_obershelp.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/ratcliff_obershelp.rs) | Gestalt pattern matching | `SimilarityMetric` | PASSED (10k) |
| **Token** | `Jaccard` | [`jaccard.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/jaccard.rs) | Jaccard multiset / set similarity | `SimilarityMetric` | PASSED (10k) |
| | `Overlap` | [`overlap.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/overlap.rs) | Overlap coefficient | `SimilarityMetric` | PASSED (10k) |
| | `Cosine` | [`cosine.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/cosine.rs) | Cosine / Ochiai coefficient | `SimilarityMetric` | PASSED (10k) |
| | `Tanimoto` | [`tanimoto.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/tanimoto.rs) | Logarithmic Tanimoto similarity | `SimilarityMetric` | PASSED (10k) |
| | `Sorensen` | [`sorensen.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/sorensen.rs) | Sorensen-Dice coefficient | `SimilarityMetric` | PASSED (10k) |
| | `Tversky` | [`tversky.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/tversky.rs) | Asymmetric Tversky index | `SimilarityMetric` | PASSED (10k) |
| | `Bag` | [`bag.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/bag.rs) | Multiset difference distance | `DistanceMetric` | PASSED (10k) |
| **Phonetic** | `MRA` | [`mra.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/mra.rs) | Match Rating Approach | `SimilarityMetric` | PASSED (10k) |
| | `Editex` | [`editex.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/editex.rs) | Phonetic-group Editex distance | `DistanceMetric` | PASSED (10k) |
| **Compression** | `RleNcd` | [`rlencd.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/rlencd.rs) | Run-Length Encoding NCD | `SimilarityMetric` | PASSED (10k) |
| | `ArithNcd` | [`arith_ncd.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/arith_ncd.rs) | Arithmetic Coding NCD | `SimilarityMetric` | PASSED (10k) |
| | `SqrtNCD` | [`sqrt_ncd.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/sqrt_ncd.rs) | Square-Root NCD | `SimilarityMetric` | PASSED (10k) |

---

## Local Building & Testing

### 1. Build the Rust Crate
```bash
cd textdistancerust
cargo build --release
```

### 2. Run the Test Suite (89 Tests)
```bash
cargo test
```
*Executes 60 unit tests and 29 known-value integration tests.*

---

## Differential Fuzz Harness

The project includes a Python Hypothesis differential fuzz harness ([`fuzz-harness/fuzz_driver.py`](file:///d:/my%20study/Project/Text_Distance_Rust/fuzz-harness/fuzz_driver.py)) that streams randomized inputs to `textdistancerust-cli` via JSON-IPC over stdin/stdout.

To execute differential fuzzing:
```bash
python fuzz-harness/fuzz_driver.py --alg hamming,jaccard,damerau_levenshtein --iterations 10000
```

---

## Docker Setup

A Docker environment with pre-installed Rust and Python toolchains is provided:

```bash
# Build Docker image
docker build -t textdistancerust:latest .

# Run tests in Docker
docker compose run --rm test

# Run differential fuzzing in Docker
docker compose run --rm fuzz python3 fuzz-harness/fuzz_driver.py --alg hamming --iterations 1000

# Run benchmarks in Docker
docker compose run --rm benchmark
```

---

## Repository Structure

```text
.
├── Dockerfile                      # Dual Rust+Python Docker image
├── docker-compose.yml              # Container services (test, fuzz, benchmark, cli)
├── requirements.txt                # Python dependencies (textdistance, hypothesis)
├── .dockerignore                   # Docker context rules
├── benchmark.py                    # Python vs Rust benchmark script
├── textdistancerust/               # Rust Crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Public library API re-exports
│       ├── main.rs                 # Persistent JSON-IPC CLI binary
│       ├── traits.rs               # DistanceMetric & SimilarityMetric traits
│       ├── tokenizer.rs            # Char, word, and n-gram tokenization
│       ├── bin/
│       │   ├── tdcli.rs            # Interactive CUI binary
│       │   ├── bench_native.rs     # Native benchmark binary
│       │   └── integration_tests.rs# 29 known-value integration tests
│       └── *.rs                    # Individual algorithm implementations
├── fuzz-harness/
│   └── fuzz_driver.py              # Differential fuzz harness (Hypothesis)
├── artifacts/
│   └── benchmark_report.md         # Generated latency benchmark report
├── README.md                       # Project documentation
├── DECISIONS.md                    # Parity decisions & scope log
├── PROJECT_RULES.md                # Hard constraints and guidelines
└── ROADMAP.md                      # Milestone breakdown
```

---

## License

This project is open-source under the [MIT License](LICENSE).
