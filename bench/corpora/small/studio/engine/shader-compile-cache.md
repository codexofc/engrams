---
name: shader-compile-cache
description: Shader variants compiled offline into a cache keyed by source hash and defines, 1 840 variants, runtime fallback logs missing ones
type: project
status: active
verified: 2026-05-26
---

## Problem

Marée has 1 840 shader variants (material features times light setups times platform). Compiling them at startup took 90 s on the low-end target and produced hitches when a new variant appeared in the world.

## Design

- The asset pipeline compiles every variant listed in `shaders/variants.toml` into a cache file per platform, keyed by `blake3(source) + defines`. The cache ships with the game.
- At runtime, `ShaderCache::get(key)` returns the compiled blob or, if missing, schedules a compile on a worker thread and returns a fallback shader (flat magenta in dev, the nearest cached variant by define distance in release). Missing keys are written to `missing_variants.log`.
- The pipeline reads `missing_variants.log` files collected from playtests and adds the variants to `variants.toml` automatically; a review sees them as a diff.

## Numbers

- Startup shader work on the low-end target: 90 s to 0.6 s (loading the cache).
- Missing variants per playtest session: 40 in March 2026, 3 in May, 0 in the last two sessions.
- Cache size: 210 MB for the desktop platform, 140 MB for the low-end target.

## Caveats

- A change in a shader include invalidates every variant that includes it; the pipeline recompiles them all (about 4 minutes on the CI). It is by design, a partial invalidation was tried and missed cases.
- The define distance fallback in release picks a variant that renders something plausible but not correct; it must never be seen in a shipped build, hence the log collection. The frame graph ([[renderer-frame-graph]]) flags any fallback use in the dev overlay.
