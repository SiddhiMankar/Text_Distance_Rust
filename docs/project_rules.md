# Project Rules — Port Mortem 2026 (Python → Rust: textdistance)

## What we're building
A from-scratch Rust reimplementation of [https://github.com/life4/textdistance](https://github.com/life4/textdistance),
targeting behavioral equivalence with the Python original.

## The original repo's role: REFERENCE ONLY, never a dependency
- The Python source at "C:\Projects\Post_Mortem\textdistance" is read-only reference material. Read it to understand *what an algorithm does*, then write independent Rust
  code that does the same thing. Do not transliterate line-by-line — think
  about it as a spec, not a template to copy.
- NEVER add a Python interpreter, subprocess call to Python, PyO3 binding, or
  any other runtime dependency on the original implementation inside the
  Rust crate itself. The Rust port must run standalone with zero Python
  dependency. (Python IS allowed in a separate, clearly-labeled dev-tooling
  script for our differential fuzz harness — that's a testing tool, not part
  of the port.)
- Do not copy code verbatim from the Python source into the Rust code, even
  as commented-out reference.
- Do the Rust implementation only in a different directory named "textdistancerust".

## Hard constraints (violating these disqualifies the submission)
- All Rust code must be written within 11:30 PM 31st July 2026 to 11:30 PM 3rd August 2026 (72-hour window). Do not reuse or backport code from outside
  this window.
- NEVER modify any file under the original repo's `tests/` directory. That
  directory's contents were hashed at kickoff; the commit we're working from
  is d6a68d61088a40eef5c88191ccf79323dbf34850 — treat it as immutable. If you think a test is wrong, flag it to me — do not edit it.
- No `unsafe` blocks unless I explicitly approve one and we document why.
- Prefer safe, idiomatic Rust (Vec/slices, not raw pointers) even under time
  pressure — safety is worth more to our score than marginal speed.
- The project must build with a single command: `cargo build --release`.
  Don't introduce a build step that requires anything else to run first.

## What "behavioral equivalence" means here, concretely
- For every ported function, its output must match the Python original's
  output for the same input, within [1e-9] for floats.
- Known, documented exception: the compression-based NCD algorithms
  (BZ2NCD, LZMANCD, ZLIBNCD) may diverge from Python because Rust's
  compression crates don't guarantee byte-identical compressed sizes to
  Python's stdlib codecs. Do not try to force-match these — implement them
  correctly per-algorithm, note the expected divergence in DECISIONS.md,
  and move on. Do not spend hours chasing exact parity here.
- Every algorithm needs at least one round of differential fuzzing (see
  /fuzz-harness) before being marked done.

## Deliverables checklist
- [ ] Rust crate, builds via `cargo build --release`
- [ ] README.md
- [ ] DECISIONS.md — document every place we chose divergence, a design
      tradeoff, or scoped something out (e.g. the NCD caveat above)
- [ ] Differential fuzz harness (Python driver + Rust CLI, see /fuzz-harness)
- [ ] Benchmark report (Python vs Rust, per algorithm)
- [ ] 5-minute demo script/recording

## When in doubt
Stop and ask me rather than guessing — especially on: what counts as
"new code," how to handle a divergence you find, or anything touching
the tests/ directory.

## Additional Engineering Rules

### Public API Compatibility

- Preserve the original public API wherever practical.
- Keep function names, parameter order, and behavior compatible with the Python implementation unless a Rust-specific adaptation is necessary.
- Behavioral compatibility is more important than making the API "more Rust-like."
- Any intentional API differences must be documented in `DECISIONS.md`.

### Incremental Porting Workflow

- Never attempt to port the entire project at once.
- Port one algorithm or module at a time.
- A module is not considered complete until it has been implemented, compiled, tested, differentially fuzzed, benchmarked, and documented.

### Correctness Before Performance

- Prioritize behavioral equivalence over optimization.
- Do not rewrite or optimize an algorithm until its behavior matches the Python implementation.
- A slower but behaviorally identical implementation is preferred over a faster implementation with incorrect behavior.

### Failure Behavior

- Behavioral equivalence includes error handling.
- Match the Python implementation's handling of invalid inputs, edge cases, exceptions, and special return values wherever practical.
- Never silently accept invalid input that the original implementation rejects.

### Build Quality

- Never leave the repository in a non-compiling state.
- Every completed change should successfully pass:
  - `cargo fmt`
  - `cargo clippy`
  - `cargo test`

### Module Organization

- Keep modules small and focused.
- Prefer one algorithm per source file.
- Group shared functionality into common utility modules rather than duplicating code.

### Source of Truth

- If the original behavior is unclear:
  1. Read the Python implementation.
  2. Read the corresponding tests.
  3. Treat the implementation and tests as the specification.
- Never invent or "improve" behavior without explicit approval.

### Dependencies

- Prefer the Rust standard library whenever practical.
- Only introduce external crates when they provide a clear implementation benefit.
- Avoid unnecessary dependencies for convenience alone.

### Benchmarking

- Always benchmark identical workloads between the Python implementation and the Rust implementation.
- Record benchmark methodology, hardware information, and results.

### Documentation

- Document every intentional behavioral difference, implementation trade-off, limitation, or design decision immediately in `DECISIONS.md` rather than postponing documentation until the end.

### Rust Code Style

- Prefer safe, idiomatic Rust.
- Prefer iterators, slices, traits, `Option`, and `Result`.
- Avoid `unwrap()`, `expect()`, and `panic!()` outside of tests unless absolutely necessary and explicitly justified.

### AI Coding Workflow

- Never generate or review multiple algorithms in a single step.
- Complete one algorithm fully before beginning the next.
- Keep generated changes small, reviewable, and easy to test.

## Definition of Done

An algorithm or module is complete only when all of the following are true:

- The Rust implementation compiles successfully.
- `cargo fmt` passes.
- `cargo clippy` passes without warnings.
- All relevant tests pass.
- Differential fuzzing shows no unexpected behavioral differences.
- Benchmarks execute successfully.
- Any implementation decisions or deviations are documented in `DECISIONS.md`.