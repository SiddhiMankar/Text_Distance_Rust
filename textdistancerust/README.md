# `textdistancerust`

A high-performance, standalone, safe Rust implementation of sequence distance metrics, matching Python's `textdistance` library with 100% behavioral equivalence.

## Features
- **Zero `unsafe` code** (`#![forbid(unsafe_code)]`).
- **Zero Python runtime dependencies**.
- **Generic sequence traits** (`DistanceMetric<T>` & `SimilarityMetric<T>`).
- **20 algorithms implemented** and fuzz-verified across 10,000+ iterations each.

## Quick Start

Add `textdistancerust` to your `Cargo.toml`:

```toml
[dependencies]
textdistancerust = { path = "../textdistancerust" }
```

### Usage Example

```rust
use textdistancerust::{Jaccard, SimilarityMetric, to_char_vec};

fn main() {
    let metric = Jaccard::new();
    let s1 = to_char_vec("hello");
    let s2 = to_char_vec("help");

    let sim = metric.similarity(&s1, &s2).unwrap();
    let dist = metric.distance(&s1, &s2).unwrap();

    println!("Similarity: {}, Distance: {}", sim, dist);
}
```

## Running Tests & Clippy

```bash
cargo test
cargo clippy --all-targets
cargo fmt -- --check
```
