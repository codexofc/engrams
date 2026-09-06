---
name: coding-conventions-rust
description: Rust conventions: rustfmt house config, clippy pedantic minus 6, no unwrap outside tests, error enums per crate, house lints
type: reference
status: active
verified: 2026-01-28
---

## Formatting and lints

- `rustfmt` with `rustfmt.toml` at the workspace root (max width 110, imports grouped std / external / crate). Not negotiable, the CI checks it.
- `clippy` at the pedantic level minus six lints listed in `clippy.toml` with the reason for each (the `module_name_repetitions` one is the most argued, kept off because engine types are named `RenderPass`, `RenderTarget`, on purpose).
- House lints, run by the `brumelint` binary in the code lane: `no-platform-cfg` (only `givre-platform` may use `cfg(target_os)`), `no-literal-ui-string` (no string literal passed to a UI function), `unsafe-needs-invariant` (an `unsafe` block must be preceded by a comment starting with `// SAFETY:` that is longer than 20 characters).

## Errors and panics

- No `unwrap()` or `expect()` outside tests and `build.rs`. Engine code returns `Result<T, CrateError>` with one error enum per crate; the game crate uses a single `GameError` that wraps them.
- Panics are for programmer errors caught in dev (`debug_assert!` on invariants); in release, the same paths return errors and log.
- An `Option` returned to mean "not found" is fine; an `Option` returned to mean "failed" is not, that is a `Result`.

## Naming

- Types `CamelCase`, functions and fields `snake_case`, constants `SCREAMING_SNAKE`, the usual. Crate names `givre-<area>`.
- A type that owns a GPU or OS resource ends with `Handle` if it is a cheap copyable id and with nothing if it owns the resource; `TextureHandle` versus `Texture`.
- No `Manager`, `Helper`, `Util` in a type name. If it manages textures, it is a `TexturePool` or a `TextureStreamer`, which says what it does.

## Tests

- Unit tests next to the code, integration tests in `tests/`, frame tests (playing frames of a reference scene) in the `maree-tests` crate.
- A test that needs a GPU is marked `#[ignore]` and run by the CI lane on the runners with a GPU only.

Commit and review rules are in [[commit-and-review-rules]]; asset naming, which the pipeline enforces, in [[naming-assets]].
