# AGENTS.md — Coding Guide for poker-rs

Scope: Applies to the entire repository rooted at this folder.

Purpose: This file tells agents and contributors how to structure, write, test, and review Rust code for this project. Follow these rules for any file you touch in this tree.

## Project Overview

- Goal: High-performance, well-tested poker hand evaluation library with a Ratatui-based TUI front-end.
- Targets: Library crate for evaluation logic; binary for TUI UI. Clean separation so the library is reusable without the TUI.

## Toolchain & Baseline

- Rust: Use latest stable via rustup. Maintain compatibility with reasonably recent stable (MSRV ~1.70+). If raising MSRV, document in PR and update CI when added.
- Edition: Prefer `2021`. When moving to a newer edition, include a dedicated PR that only updates edition + fixes.
- rustfmt: Required. Run `cargo fmt --all` before committing.
- Clippy: Required. Run `cargo clippy --all-targets --all-features -- -D warnings` locally. Treat warnings as errors.
- OS/Arch: Must build and test on macOS and Linux x86_64. Avoid OS-specific assumptions in the library.

## Repository Layout (expected)

- Library crate: `src/lib.rs` with modules for core logic (e.g., `cards/`, `hand/`, `evaluator/`, `deck/`).
- TUI binary: `src/bin/poker.rs` (or `src/main.rs`) with TUI code under `src/tui/` (views, state, input). Keep UI strictly separated from core.
- Integration tests: `tests/` (black-box tests of the public API).
- Examples: `examples/` (runnable samples; also used in docs and CI sanity checks).
- Benchmarks: `benches/` (Criterion-based micro/meso benchmarks for evaluator hot paths).

Do not create extra top-level folders unless justified. Prefer smaller modules over monoliths. Keep file names `snake_case.rs`.

## Cargo Conventions

- Profiles (to add in `Cargo.toml` when crate exists):
  - `[profile.release] opt-level = 3, lto = "thin", codegen-units = 1, strip = true`
  - `[profile.dev] opt-level = 1` (keep debug builds reasonably fast)
- Features:
  - `serde` (optional) for serialization of types if/when needed.
  - `bench` (optional) to gate heavy benchmarking dependencies from normal builds.
  - Keep default features minimal; do not enable features that pull large dep trees by default.
- Dependency policy:
  - Keep the dependency graph lean. Justify any new dependency in the PR description.
  - Favor well-maintained crates with permissive licenses.
  - Avoid unsafe code and FFI unless strictly necessary and accompanied by safety docs and tests.

## Code Style & API Design

- Naming:
  - Crates and modules: `snake_case`
  - Types/Traits/Enums/Structs: `UpperCamelCase`
  - Functions/Methods/Vars: `snake_case`
  - Constants/Statics: `SCREAMING_SNAKE_CASE`
- Visibility:
  - Keep the public API small and clear. Prefer `pub(crate)` over `pub` where possible.
  - Mark enums and structs `#[non_exhaustive]` when future variants/fields are likely.
- Panics:
  - Avoid panics in library code. Use `Result` for recoverable errors. Panics are acceptable in tests and in the TUI `main()` when clearly user-facing and explained.
- Error handling:
  - Library: `thiserror` for typed errors where appropriate. Avoid a single catch-all enum for unrelated domains.
  - Binary/TUI: `anyhow` for ergonomic error propagation at edges.
  - Never leave `unwrap`/`expect` in library or shared modules; use them only in tests or clearly non-failing invariants documented with `debug_assert!`.
- Zero-cost abstractions:
  - Use iterators and slices; avoid unnecessary allocations and cloning. Clone only at ownership boundaries.
  - Prefer `Copy` small value types (e.g., card representations) where ergonomic.
- Docs:
  - Every public item gets `rustdoc` comments. Provide examples that compile (doc tests) for the main API entry points.
  - Add crate-level docs in `lib.rs` explaining design, examples, and performance notes.

## Module Guidance (Library)

- cards: Card representation (rank/suit), parsing/format, ordering. Keep compact (e.g., bit-packed) if it materially helps performance and remains readable.
- hand: Data types for player hands and board cards. Include validation helpers.
- evaluator: Hand ranking, comparisons, tie-breakers. Provide pure functions plus a stateful facade if needed for caching.
- variants: Optional module for non–Texas Hold’em (Omaha, etc.) behind feature flags once stable.
- deck: Shuffling and dealing utilities, reproducible RNG hooks (seeded PRNG for tests/benches).

Keep evaluator logic deterministic and easy to test with property-based tests.

## Testing Strategy

- Unit tests: Colocated in each module (`mod tests`) for small, focused cases.
- Integration tests: `tests/` to exercise public API and cross-module behavior.
- Property-based tests: Use `proptest` for evaluator correctness (e.g., rank invariants, stability under permutations, tie-break orderings).
- Doc tests: Keep runnable examples up to date.
- Coverage (optional): `cargo tarpaulin` locally. Do not gate PRs on numeric coverage, but require meaningful tests for new logic.

Run locally:

```
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Benchmarking & Performance

- Use `criterion` benchmarks in `benches/` focused on evaluator hot paths.
- Check performance regressions before merging changes to evaluator/critical code.
- Use `#[inline]` only after demonstrating wins in benchmarks.
- Prefer value types and slices over heap allocations; avoid `String` when `&str`/`&[u8]` suffices.

## Documentation

- Keep `README.md` concise; point to crate docs for API details.
- Examples in `examples/` should compile and run; mirror key doc examples there.
- If adding algorithms, include brief rationale/complexity notes in docs and/or `doc/`.

## Security & Correctness

- Validate inputs at public boundaries; document assumptions.
- Avoid `unsafe`. If unavoidable, isolate it in a dedicated module with clear safety comments and exhaustive tests.
- No panics in library for ordinary invalid input; return errors.

## CI (when added)

Minimum checks to run on PRs/branches:

```
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --all-features
```

Add `cargo doc --no-deps -D warnings` if docs are established. Consider a `bench` job triggered manually or on schedules.

## Contribution & Review

- Small, focused PRs. Include tests and docs for new behavior.
- Commit messages: imperative mood, short summary line (<72 chars), details in body if needed.
- Do not bundle refactors with feature changes unless essential.
- Explain any new dependency and its impact (size, MSRV, maintenance).

## Agent-Specific Rules (Codex)

- Before coding:
  - Scan `tasks.md` to align with milestones.
  - If the crate does not exist yet, propose a minimal Cargo scaffold in your response before creating it (unless explicitly asked to scaffold).
- When editing code:
  - Follow rustfmt/clippy guidance above. Keep public API minimal.
  - Add/adjust tests alongside changes; include property tests for evaluator logic where feasible.
  - Avoid adding dependencies unless necessary; justify in the PR/response.
  - Keep patches minimal and focused; do not fix unrelated issues.
- Validation:
  - Run `cargo fmt`, `cargo clippy`, and `cargo test` locally before concluding work when a Cargo project exists.
- Documentation:
  - Update `rustdoc` for changed public items and add/refresh examples.

## Getting Started (once scaffolded)

Common commands:

```
rustup default stable
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run --bin poker-rs   # if TUI binary exists
```

Optional tooling:

- `cargo-udeps` to detect unused dependencies.
- `cargo-audit` to check for vulnerable crates.
- `cargo-insta`/`insta` for snapshot tests where UI output is involved.

---

Questions or ambiguities: Prefer asking for clarification in the PR/issue. If a decision is needed, choose the simplest approach that keeps the public API clean and the dependency tree small.
