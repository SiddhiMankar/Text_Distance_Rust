# 50-Hour Rust Porting & Differential Fuzzing Roadmap: `textdistancerust`

## Project Overview
This document outlines the architecture, milestone breakdown, and continuous verification strategy for porting [`textdistance`](file:///c:/Projects/Post_Mortem/textdistance/textdistance/__init__.py) (Python) to standalone Rust (`textdistancerust`). 

The hackathon judging model allocates **70% of points to Functionality, Reliability, and Behavioral Equivalence**. This roadmap prioritizes **continuous differential fuzzing on every algorithm upon landing** over unverified API breadth.

---

## 1. Hard Constraints Summary (`PROJECT_RULES.md`)

1. **Strict Timeframe Scope**: All Rust code must be written within the designated 72-hour window; no pre-existing code reuse or external backporting is allowed.
2. **Immutable Original Test Suite**: The contents of the `tests/` directory under `textdistance` are hashed and read-only (`d6a68d61088a40eef5c88191ccf79323dbf34850`). Modifying test files to force passes is strictly prohibited. This commit hash (`d6a68d61088a40eef5c88191ccf79323dbf34850`) was published by the organizers at kickoff and cross-checked locally against `git rev-parse HEAD` in the repository root at project start.
3. **Standalone Rust Crate (Zero Runtime Python)**: The compiled binary/crate `textdistancerust` must run standalone with zero runtime dependency on Python, PyO3, or subprocess calls. Python is permitted strictly as a separate test driver inside the differential fuzz harness.
4. **Zero Unsafe Code Policy**: No `unsafe` blocks are permitted in the codebase unless explicitly approved by the lead and documented in `DECISIONS.md`.
5. **Single-Command Release Build & Mandatory Deliverables**: Must build via a single `cargo build --release` command and pass `cargo fmt`, `cargo clippy`, unit tests, differential fuzzing, while delivering `README.md`, `DECISIONS.md`, benchmark reports, and a 5-minute video demo.

---

## 2. Blocking Day-1 Prerequisites (Hours 0 – 3)

Before any algorithm implementation begins, the following 3 architectural decisions must be locked in code:

### Architectural Locks

> [!IMPORTANT]
> **Prerequisite 1: Generic Trait Abstraction**
> Define `DistanceMetric<T>` and `SimilarityMetric<T>` traits in `textdistancerust::traits` to handle generic sequences (`&[T]`, `&str`, `Vec<char>`).
>
> ```rust
> pub trait DistanceMetric<T> {
>     fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError>;
>     fn maximum(&self, s1: &[T], s2: &[T]) -> f64;
>     fn normalized_distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
>         let max = self.maximum(s1, s2);
>         if max == 0.0 { return Ok(0.0); }
>         Ok(self.distance(s1, s2)? / max)
>     }
> }
> ```
>
> > [!NOTE]
> > **Empty-Input & Zero-Max Parity Requirement**
> > The default `Ok(0.0)` in `normalized_distance` is a fallback only. Python's actual behavior when `maximum() == 0` (e.g. both inputs empty) is not uniform across algorithms — some raise `ZeroDivisionError`, some return `0.0`, and some return `nan`. Each algorithm's implementer must check the real Python source for that specific algorithm's empty-input behavior and override the trait default if it differs. "Confirmed against Python's empty-input/zero-max behavior" is a required checkbox in every algorithm's Definition of Done, not something the trait is allowed to paper over.

> [!IMPORTANT]
> **Prerequisite 2: Configuration & Tokenization Builder Pattern**
> Standardize algorithm structs to contain optional configuration fields (`qval: Option<usize>`, `as_set: bool`, `restricted: bool`, `gap_cost: f64`). Provide a unified `TokenizedSequence<T>` helper to slice strings by UTF-8 characters (`Vec<char>`), words, or $q$-grams.

> [!IMPORTANT]
> **Prerequisite 3: Unified Error Handling Convention**
> Establish `TextDistanceError` enum (`InvalidParameter`, `EmptyInputSequence`, `CalculationOverflow`, `IncompatibleLength`) to handle divide-by-zero, invalid range parameters, and domain bounds cleanly without panics (`unwrap`/`expect`).

---

## 3. Differential Fuzz Harness Architecture (Built in Hours 1 – 3)

To achieve continuous fuzz-verification, the differential fuzzing pipeline is established **before algorithm porting begins**.

```mermaid
graph LR
    Subgraph Harness ["Differential Fuzzing Harness (Persistent IPC Protocol)"]
        Fuzzer["Python Driver (Hypothesis + Seed Corpus)"]
        RPC["Persistent JSON Stream (stdin/stdout)"]
        RustCLI["`textdistancerust-cli` Long-Lived Executable"]
    end

    Fuzzer -->|"1. Stream JSON `{alg: 'levenshtein', s1: '...', s2: '...'}`"| RPC
    RPC -->|"2. Process line via stdin"| RustCLI
    Fuzzer -->|"3. Compute Reference in `textdistance` (Python)"| RefResult["Python Output"]
    RustCLI -->|"4. Emit JSON result to stdout"| RustResult["Rust Output"]
    Fuzzer -->|"5. Assert |Python - Rust| < 1e-9"| MatchCheck{"Match?"}
    MatchCheck -- "Mismatch" --> BugReport["Log Failure & Repro Case to Fuzz Log"]
    MatchCheck -- "Success" --> PassLog["Increment Survivor Count"]
```

> [!IMPORTANT]
> **Persistent Process Architecture**:
> `textdistancerust-cli` runs as a **long-lived persistent process**, reading one JSON test case per line from `stdin` and writing one JSON result per line to `stdout` in a continuous streaming loop — **not spawned fresh per test case**. At $\sim 1,000,000+$ planned fuzz iterations, per-case process spawn overhead would add tens of minutes of OS process creation latency before any computation occurs. The Python driver spawns the Rust executable once at harness startup and streams cases continuously.

---

## 4. Phase-by-Phase 50-Hour Execution Roadmap

### Phase 0: Setup, Day-1 Prerequisites & Harness (Hours 0 – 3)
* **Goal**: Scaffold crate, lock architecture traits, build persistent streaming differential fuzz harness driver & CLI.
* **Work Allocation**:
  * **Person A (Lead)**: Initialize `textdistancerust` workspace, write `traits.rs`, `error.rs`, and establish persistent stdin/stdout CLI JSON protocol.
  * **Person B**: Build `tokenizer.rs` (UTF-8 `Vec<char>`, word split, $q$-gram generator).
  * **Person C**: Construct Python Hypothesis differential fuzz driver in `fuzz-harness/`, adding a seed generator for original `tests/*.py` cases as well as explicit empty `""` and single-character input cases.
* **Definition of Done**:
  1. `cargo test` passes cleanly.
  2. Running `python fuzz-harness/fuzz_driver.py` communicates with `textdistancerust-cli` via stdin/stdout and successfully verifies dummy `Identity` metric results.
  3. **Persistent Harness Performance**: Verified by timing 10,000 trivial `Identity` calls and confirming throughput isn't dominated by process-start overhead.
  4. **Seed Corpus Pass**: Harness's first verification pass runs every literal test case from the original `textdistance/tests/` directory (the hashed, read-only suite) against both implementations before any Hypothesis-generated random input is used. Failures here are treated as confirmed bugs and fixed immediately.

---

### Phase 1: Core Algorithms & Initial Differential Fuzzing (Hours 3 – 12)
* **Goal**: Port fundamental edit, token, simple, and matrix metrics; fuzz every metric upon completion.
* **Work Allocation**:
  * **Person A (Edit)**: Implement `Hamming` (with `truncate`), `Levenshtein` (row-cycling $O(M)$ DP), `DamerauLevenshtein` (restricted OSA mode).
  * **Person B (Token, Simple & Matrix)**: Implement `Prefix`, `Postfix`, `Length`, `Identity`, `Matrix` (customizable substitution-matrix comparator), `Jaccard`, `Overlap`, `Cosine`, `Tanimoto` (log-based token similarity — distinct formula from Jaccard despite superficial similarity, do not alias them).
  * **Person C (Sequence & Benchmarks)**: Implement `LCSSeq` (2D matrix DP), `LCSStr` (n-gram scanning), initialize benchmark harness.
* **Continuous Fuzz-Verification**:
  * Run `fuzz_driver.py --alg hamming,levenshtein,jaccard,lcsseq,matrix,tanimoto` with **10,000+ iterations per algorithm minimum**.
* **Definition of Done**: All Phase 1 algorithms compile clean (`cargo clippy` 0 warnings), pass unit tests, pass empty-input/zero-max parity checks against Python source, and complete 10,000+ differential fuzz iterations with 0 mismatches.

---

### Phase 2: Moderate Complexity Metrics & NCD Initial Pass (Hours 12 – 24)
* **Goal**: Implement moderate edit metrics, set-theory metrics, gestalt pattern matching, and exact NCD algorithms.
* **Work Allocation**:
  * **Person A (Edit)**: Implement `Jaro`, `JaroWinkler` (prefix boost & long tolerance), `DamerauLevenshtein` (unrestricted mode).
  * **Person B (Token & Phonetic)**: Implement `Sorensen`, `Tversky` (asymmetric weighting & bias), `Bag`, `MRA` (Match Rating Approach).
  * **Person C (Sequence & Compression)**: Implement `RatcliffObershelp`, `ArithNCD` (exact big-rational math via `num-rational`), `RLENCD`, `BWTRLENCD`.
* **Continuous Fuzz-Verification**:
  * Fuzz `JaroWinkler`, `Tversky`, `MRA`, `RatcliffObershelp`, and `ArithNCD` against Python reference implementation — **10,000+ iterations per algorithm minimum**, consistent with Phase 1's standard.
* **Definition of Done**: Intermediate metrics verified by fuzz harness (10k+ iterations each); exact-match NCD metrics (`ArithNCD`, `RLENCD`, `BWTRLENCD`) match Python outputs within $1\text{e-}9$; empty-input parity verified per algorithm.

---

### Phase 3: Advanced Alignment, Phonetic & Compression Pivot Checkpoint (Hours 24 – 36)

> [!WARNING]
> **Midpoint Checkpoint & Compression Pivot (Hour 24 – 26)**
> * Implement `BZ2NCD`, `LZMANCD`, `ZLIBNCD` using Rust native compression crates (`flate2`, `bzip2`, `lzma-rs`).
> * Execute differential fuzzing to identify crate header/dictionary output variances between Rust and Python stdlib codecs.
> * **PIVOT**: As mandated by `PROJECT_RULES.md`, cap time spent on codec byte-parity at 2 hours. Document confirmed cross-language compression codec size divergences in `DECISIONS.md` and lock implementations.

* **Work Allocation**:
  * **Person A (Edit/Alignment)**: Implement `NeedlemanWunsch` (global alignment), `SmithWaterman` (local alignment), `Gotoh` (affine gap alignment — separate gap-open/gap-extend costs, distinct from NW/SW), `MLIPNS`.
  * **Person B (Phonetic/Edit)**: Implement `Editex` (grouped phonetic letter sets `{AEIOUY}`, `{BP}`, etc., and `ungrouped` `{HW}`), `StrCmp95`.
  * **Person C (Advanced Token/Compression)**: Implement `MongeElkan` (delegating to inner `DamerauLevenshtein`), `SqrtNCD`, `EntropyNCD`, complete compression pivot documentation.
* **Continuous Fuzz-Verification**:
  * Fuzz `Gotoh`, `Editex`, `StrCmp95`, `NeedlemanWunsch` with complex Unicode test vectors and 10,000+ iteration floors.
* **Definition of Done**: High-complexity metrics pass 10,000+ differential fuzz cases; empty-input behaviors match Python; `DECISIONS.md` updated with NCD divergence rationale.

---

### Phase 4: Full Suite Fuzzing Campaign & Benchmark Execution (Hours 36 – 44)
* **Goal**: Run large-scale differential fuzzing campaign across all algorithms, execute performance benchmarks, enforce zero-clippy/zero-fmt warnings.
* **Work Allocation**:
  * **Person A**: Run 500,000-iteration differential fuzzing campaign over persistent IPC. Fix any edge-case divergence surfaced by Hypothesis.
  * **Person B**: Code cleanup, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, refactor shared utility logic.
  * **Person C**: Execute benchmark suite (`Criterion` / benchmark runner) comparing Python vs Rust throughput across sequence lengths ($10, 100, 1000$).
* **Definition of Done**:
  * Differential fuzzing campaign completes **1,000,000 total test cases with 0 unhandled failures/mismatches**.
  * Benchmark report generated containing throughput tables and charts.

---

### Phase 5: Audit, Deliverables Polish & Code Freeze (Hours 44 – 50)
* **Goal**: Perform safety audit, finalize documentation, record 5-minute video demo, execute final release build.
* **Work Allocation**:
  * **Person A**: Conduct **Unsafe Code Audit** (verifying 0 `unsafe` blocks exist), finalize `fuzzing_report.md` / Bug Catcher writeup.
  * **Person B**: Write `README.md` (installation, quickstart, architecture summary) and finalize `DECISIONS.md`.
  * **Person C**: Record 5-minute video demo script and presentation walk-through; perform final `cargo build --release` check.
* **Definition of Done**: `cargo build --release` succeeds cleanly; 0 `unsafe` blocks present; `README.md`, `DECISIONS.md`, benchmark data, fuzz survivor report, and demo video ready for submission.

---

## 5. Midpoint Realignment & Scope Cut Line (Hour 24)

If execution falls behind schedule at the Hour 24 midpoint, scope will be shed in the following strict priority order to **protect differential fuzzing and behavioral equivalence of core algorithms over breadth**:

```
[DROP FIRST]  1. Vector-based draft metrics (Chebyshev, Minkowski, Euclidean, Correlation)
      │       2. Gotoh affine gap alignment (preserve Needleman-Wunsch & Smith-Waterman)
      │       3. StrCmp95 OCR matrix (preserve JaroWinkler)
      ▼       4. Limit Compression family strictly to ArithNCD, RLENCD, SqrtNCD
[PROTECT AT   Core Edit (Hamming, Levenshtein, DamerauLevenshtein, JaroWinkler),
 ALL COSTS]   Sequence (LCSSeq, LCSStr), Token (Jaccard, Sorensen, Cosine, Bag, Matrix, Tanimoto)
```

---

## 6. Proactive Shortcut & Rule Violation Prevention Matrix

| Tempting Shortcut | Rule Violated | Prevention Mechanism in Roadmap |
| :--- | :--- | :--- |
| Invoking Python via PyO3 inside `textdistancerust` for hard metrics | Rule 2.3 (Zero Runtime Python) | Rust crate builds standalone with zero PyO3/Python dependencies. Python is restricted to `fuzz-harness/` external testing scripts. |
| Editing original `tests/*.py` files to ignore failing test cases | Rule 2.2 (Immutable Test Suite) | Original `tests/` directory is treated as read-only. All bug fixes must occur inside `textdistancerust/`. |
| Adding `unsafe` pointer slicing for speed gains | Rule 2.4 (Zero Unsafe Code) | Enforce `#![forbid(unsafe_code)]` in `textdistancerust/src/lib.rs`. |
| Postponing differential fuzzing until the final 6 hours | Verification First Policy | Fuzz harness is built in Phase 0; every algorithm is fuzzed immediately upon implementation. |
| Copy-pasting Python comments/code verbatim into Rust | Rule 2.1 (Reference Only) | Rust code is written independently based on algorithmic specifications. |

---

## 7. Verification & Deliverables Matrix

| Deliverable | Location / Target | Responsible Owner | Verification Method |
| :--- | :--- | :--- | :--- |
| **Standalone Rust Crate** | `textdistancerust/` | Team | Single command `cargo build --release` |
| **Differential Fuzz Harness** | `fuzz-harness/` | Person C / Person A | Automated persistent stdin/stdout streaming Hypothesis driver ($< 1\text{e-}9$) |
| **Fuzz Survivor Report** | `artifacts/fuzzing_report.md` | Person A | Summary of 1,000,000+ fuzzing iterations and edge case resolutions |
| **Decision & Divergence Log** | `DECISIONS.md` | Person B | Documentation of NCD codec behavior, float tolerance, empty-input parity, and trait designs |
| **Benchmark Report** | `artifacts/benchmark_report.md` | Person C | Criterion benchmark comparisons across sequence lengths |
| **README & Demo Video** | `README.md`, `demo.mp4` | Person B / Person C | Presentation walkthrough and instructions |
