---
name: ci-pipeline-layout
description: Three CI lanes (code per merge request, assets nightly, playtest on tag) on 4 self-hosted runners, 11 minutes for a code lane
type: reference
status: active
verified: 2026-06-01
---

## Lanes

1. **Code** (every merge request and every merge to `main`): format, clippy with the house lints, build in dev and release for desktop, unit tests, the frame tests (`no_alloc_in_hot_systems`, the 40 reference captures compared with a tolerance). 11 minutes with a warm cache, 25 cold. A merge request cannot be merged with a red code lane.
2. **Assets** (nightly at 02:00, and on demand from the editor's "request CI build" button): `brumepipe build` for both platforms, then the pack verification. 6 minutes on a typical night, 41 on a full rebuild ([[asset-pipeline-overview]]). Failures are posted in the art channel with the asset name and the `brumepipe why` output.
3. **Playtest** (on a `playtest/*` tag): release build for both platforms, assets, symbols uploaded to the build server, installer, and the smoke test that launches the game on the two reference machines and plays a 2-minute replay. 55 minutes.

## Runners

Four self-hosted runners in the studio's machine room: two 32-core machines for code and assets, one desktop reference machine and one low-end target dev kit for the smoke test. The old cloud runners are described in [[ci-old-runners]].

Runner disks hold the compilation cache (shared with dev machines) and the asset cache; both are on the studio network storage, not on the runner disks, since the cache misses episode of [[ci-cache-misses-lesson]].

## Rules

- No lane longer than 60 minutes; a step that pushes a lane past it is split or moved to nightly.
- Flaky tests are quarantined by a label within a day and fixed within a week, or deleted. The reference capture test was flaky for two weeks in April 2026 because of a driver update on the desktop runner; the tolerance was raised for that capture set and the driver pinned.
- The CI does not deploy anything. Playtest builds are picked up by hand from the build server.
