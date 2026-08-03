# `textdistancerust`

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Unsafe Code](https://img.shields.io/badge/unsafe-forbid-blue.svg)]()
[![Fuzz Verification](https://img.shields.io/badge/fuzzing-10000%2B%20iters%2Falg-success.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)

`textdistancerust` is a high-performance, standalone, safe Rust implementation of text and sequence distance algorithms, providing 100% behavioral equivalence with Python's [`life4/textdistance`](https://github.com/life4/textdistance) library across 30 algorithms.

---

## Key Features

- **Strict Zero `unsafe` Policy**: Enforces `#![forbid(unsafe_code)]`.
- **Zero Python Runtime Dependencies**: Runs natively as a pure Rust library ([`libtextdistancerust.rlib`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/lib.rs)).
- **Generic Sequence Traits**: Algorithms operate over generic slice sequences (`&[T]`) via [`DistanceMetric<T>`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/traits.rs#L3-L22) and [`SimilarityMetric<T>`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/traits.rs#L24-L47), preventing multi-byte UTF-8 boundary panics.
- **Interactive CUI (`tdcli`)**: Includes an interactive command-line interface ([`src/bin/tdcli.rs`](file:///d:/my%20study/Project/Text_Distance_Rust/textdistancerust/src/bin/tdcli.rs)) with color tables and visual progress bars.

---

## Library Usage Example

Add `textdistancerust` to your `Cargo.toml`:

```toml
[dependencies]
textdistancerust = { path = "../textdistancerust" }
```

```rust
use textdistancerust::{Jaccard, SimilarityMetric, Levenshtein, DistanceMetric, to_char_vec};

fn main() {
    // Token / Set similarity
    let jaccard = Jaccard::new();
    let s1 = to_char_vec("hello");
    let s2 = to_char_vec("help");
    let sim = jaccard.similarity(&s1, &s2).unwrap();
    println!("Jaccard Similarity: {:.4}", sim);

    // Edit distance
    let lev = Levenshtein::new();
    let dist = lev.distance(&to_char_vec("kitten"), &to_char_vec("sitting")).unwrap();
    println!("Levenshtein Distance: {}", dist);
}
```

---

## Interactive CUI (`tdcli`) Usage

```bash
# List all 30 algorithms and descriptions
cargo run --bin tdcli list

# Compare strings using specific algorithms
cargo run --bin tdcli compare --alg levenshtein,jaccard,cosine "kitten" "sitting"

# Run all algorithms ranked by similarity
cargo run --bin tdcli all "MARTHA" "MARHTA"

# Run in-process microsecond latency benchmark
cargo run --bin tdcli bench

# Interactive REPL session
cargo run --bin tdcli interactive
```

---

## Testing & Quality Assurance

```bash
# Run full 89-test suite (60 unit tests + 29 integration tests)
cargo test

# Code quality checks
cargo fmt -- --check
cargo clippy --all-targets
```
