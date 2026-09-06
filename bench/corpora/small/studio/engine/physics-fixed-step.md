---
name: physics-fixed-step
description: Physics at a fixed 60 Hz with at most 4 substeps then time dropped, interpolated rendering, boat buoyancy at 120 Hz inside the step
type: reference
status: active
verified: 2026-03-17
---

Physics in Givre runs at a fixed 60 Hz regardless of frame rate. The frame accumulates elapsed time and runs as many 16.67 ms steps as fit, at most 4; beyond that, the remainder is dropped and a `physics.time_dropped` counter increments (visible in the `F3` overlay). Dropping time is the deliberate choice: a spiral of death where physics tries to catch up after a hitch made the boat teleport in the February 2026 playtest.

Rendering interpolates between the last two physics transforms with the accumulator's fraction, so a 144 Hz display sees smooth motion from 60 Hz physics. Gameplay systems that need the exact physics state (collision responses, triggers) run in the `PhysicsSync` stage of the [[ecs-scheduler]], after the steps of the frame.

## Buoyancy

The boat is the whole game, so its solver is special-cased: inside each 60 Hz step, the buoyancy and wave interaction of the player's boat runs two 120 Hz substeps. It samples the wave height field at 24 points on the hull, integrates forces, and applies damping tuned by the design team in `boat.toml` (`linear_damping`, `angular_damping`, `keel_factor`). Other floating objects use a single 60 Hz step with 4 sample points.

The 120 Hz solver was the result of a week of tuning: at 60 Hz the boat oscillated in short waves at speeds above 8 m/s, and no damping value fixed it without making the boat feel glued.

## Determinism

The step is deterministic given the same inputs and the same platform: the replay system records inputs and the physics seed and replays to the same state, which is how the QA team reproduces bugs. Cross-platform determinism is not guaranteed (floating point differences in the wave sampling); replays are per platform.

## Numbers

One 60 Hz step on the low-end target, `port_nuit` scene: 1.1 ms, of which buoyancy 0.3 ms, broadphase 0.2 ms, narrowphase and solver 0.5 ms, the rest bookkeeping. 200 dynamic bodies, 3 000 static.

## The wave field

The height field the buoyancy samples is a sum of 32 Gerstner waves computed on the GPU into `sea_displacement` (a 512 × 512 texture over a 256 m tile, repeated) and read back on the CPU for physics at a lower resolution (64 × 64) once per physics step. The readback is one frame late by design: the physics step at time `t` samples the field computed for the frame rendered at `t - 1`. At 12 m/s and 60 Hz, the boat has moved 20 cm between the two, which is under the sampling resolution of the field anyway.

The wave parameters (amplitude, wavelength, direction, steepness per wave) come from `sea/<weather>.toml`, one file per weather state, blended over 20 seconds on a weather change. The blend is linear on parameters, which produces a plausible transition; a blend on the height field itself was tried and produced interference patterns that looked like a bug.

## Collision layers

Eight layers, a 8 × 8 boolean matrix in `physics.toml`: `boat`, `world_static`, `world_dynamic`, `character`, `trigger`, `water_surface`, `debris`, `rope`. Notable pairs: `debris` does not collide with `debris` (200 pieces of driftwood in `tempete` were 40 % of the narrowphase before this), `rope` collides only with `boat` and `world_static`, `character` collides with `boat` through a dedicated contact model that keeps the character on deck when the boat pitches (the character is kinematic relative to the deck, not simulated).

## Sleeping and waking

Bodies sleep after 1 s under 2 cm/s and 1 deg/s. A body on the boat's deck is parented to the boat for sleeping purposes, so it sleeps when the boat's motion relative to the deck is low, not when the boat is still (the boat is never still). Without this rule, cargo crates never slept and cost 0.2 ms per step for nothing.

Waking is by broadphase overlap, plus an explicit wake on the boat's deck contents when the boat's angular acceleration exceeds a threshold, so a wave hit wakes the crates before they visibly clip.

## Numbers by scene, one step, low-end target

| Scene | Dynamic bodies | Step ms | Buoyancy ms |
|---|---|---|---|
| `port_nuit` | 200 | 1.1 | 0.3 |
| `tempete` | 340 | 1.9 | 0.5 |
| `baie_nord` | 120 | 0.8 | 0.3 |
| `chantier` (harbour with cranes) | 410 | 2.2 | 0.2 |

`chantier` is the only scene where the step exceeds the 2 ms budget; the crane chains are 60 bodies each and a rope model replacing them is on the list, ticket BR-421.
