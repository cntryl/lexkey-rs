# Repository Guidelines

## Project Structure & Module Organization

This is a Rust library crate published as `lexkey`. Core source lives in `src/`: `lib.rs` defines the public crate surface, macros, and shared helpers; `lexkey.rs` contains the allocating `LexKey` API; `encoder.rs` contains the reusable buffer-oriented `Encoder`. Unit tests are colocated in the source modules under `#[cfg(test)]`, and rustdoc examples act as doc-tests. Criterion benchmarks live in `benches/`, with shared benchmark configuration in `benches/common.rs`. `docs/SPEC.md` documents encoding behavior; keep behavior changes aligned with that spec and `README.md`.

## Build, Test, and Development Commands

- `cargo build` compiles the library.
- `cargo test` runs unit tests and doc-tests.
- `cargo fmt --check` verifies rustfmt formatting; run `cargo fmt` to apply it.
- `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` enforces lint cleanliness for library, tests, and benches.
- `cargo bench --bench lexkey` and `cargo bench --bench encoder` run the Criterion benchmark suites.

## Coding Style & Naming Conventions

Use standard rustfmt formatting and Rust 2021 idioms. Prefer clear, small functions over broad abstractions. Public types and traits use `UpperCamelCase` (`LexKey`, `Encoder`, `Encodable`); functions, variables, and test names use `snake_case`; constants use `SCREAMING_SNAKE_CASE`. Keep public APIs documented with rustdoc examples when practical. Fix Clippy findings directly rather than adding lint suppressions.

## Testing Guidelines

Add focused tests next to the module being changed. Existing tests use descriptive names such as `should_encode_u64_as_8_bytes` and `encode_f64_into_panics_on_nan`. For panic tests, include `#[should_panic(expected = "...")]`. When changing encoding semantics, add ordering and byte-level assertions, and update rustdoc examples or `docs/SPEC.md` if the documented contract changes.

## Commit & Pull Request Guidelines

Recent commits use short imperative or descriptive subjects such as `Optimize lexkey storage key encoding`, `clippy fixes`, and dependency bump messages from automation. Keep commit subjects concise and scoped to one change. Pull requests should describe the behavior change, note any API or encoding compatibility impact, link related issues when available, and include the commands run, especially `cargo test`, `cargo fmt --check`, and the Clippy command above. Include benchmark results when performance claims are part of the change.
