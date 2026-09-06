---
name: build-times-incremental
description: Givre incremental debug build after a one-line change is 4.1 s (was 38 s), from the crate split, dev dynamic linking and cache
type: project
status: active
verified: 2026-06-03
---

## Where we started

January 2026: a one-line change in gameplay code rebuilt for 38 seconds in debug. The engine was one crate of 210 000 lines and `cargo` recompiled it whenever anything in it changed, then relinked a 900 MB binary.

## What changed

1. **Crate split** (February): `givre-core`, `givre-render`, `givre-physics`, `givre-audio`, `givre-ecs`, `givre-platform`, and the game crate `maree`. Gameplay changes now recompile `maree` only. 38 s to 16 s.
2. **Dynamic linking in dev**: the engine crates are built as a `cdylib` once and the game links against it in dev profiles (`[profile.dev] ... `, feature `dynlink`). Relink of the game alone: 16 s to 6 s. Release builds are static, unchanged.
3. **Shared compilation cache** on the office network for third-party crates and the engine crates, so a fresh checkout does not rebuild 400 dependencies. Cold build from 14 min to 3 min 20 s when the cache is warm. The CI also feeds it, see the tools team's CI notes.
4. **Linker**: the fast linker of the platform instead of the default one, 6 s to 4.1 s.

Now: 4.1 s median for a gameplay change, 11 s for a change in `givre-render`, 3 min 20 s cold. Measured on the reference dev machine, 16 cores.

## What did not work

- `opt-level = 1` on dependencies in dev: build slightly slower, runtime much better, kept for that reason but no build gain.
- Splitting `givre-render` further: the passes depend on the same types, the split added 40 lines of re-exports for no gain.

## Rules

- No `build.rs` that touches the network or generates code from assets. Asset code generation is the pipeline's job ([[asset-streaming-budget]] explains what the runtime expects).
- A change that raises the incremental time above 6 s on the reference machine needs a note in the review. `cargo build --timings` is attached when asked.
