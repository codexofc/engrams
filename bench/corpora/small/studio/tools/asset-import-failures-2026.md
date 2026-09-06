---
name: asset-import-failures-2026
description: 1 140 import failures in H1 2026, 70 % from four causes, each now with a message that says the fix, failures halved by June
type: project
status: active
verified: 2026-06-24
---

## Figures

From January to June 2026 the pipeline ([[asset-pipeline-overview]]) logged 1 140 import failures, across 14 artists and the CI. Grouped by error code:

| Code | Count | Cause |
|---|---|---|
| `texture.not_power_of_two` | 312 | export at 1000 or 1500 pixels |
| `mesh.unnamed_node` | 226 | FBX exported with default node names, the pipeline cannot map them to sockets |
| `texture.unknown_suffix` | 168 | file saved without `_alb`, `_nrm`, ... ([[texture-compression-settings]]) |
| `texture.too_large` | 91 | source above 8192 |
| `mesh.too_many_materials` | 74 | more than 8 material slots |
| `audio.wrong_rate` | 60 | 44.1 kHz WAV instead of 48 |
| other (19 codes) | 209 | |

## What changed

Following the engine team's lesson on crash reports ([[crash-triage-lesson]]), every failure now carries: the code, a one-line fix in plain language ("resize to 1024 or 2048"), the path, and a link to the convention page. The message shows in the editor's import panel and in the art channel for CI failures.

Effect: failures per week went from 55 in January to 22 in June, and the time between a CI failure and its fix from a median of 3 hours to 40 minutes. `texture.unknown_suffix` almost disappeared (168 in the period, 4 in June) after the editor started proposing the suffix at save time.

## Decisions

- No automatic fixing (no silent resize of a 1500 texture to 1024). Artists asked for it; refused because an automatic resize hides a mistake in the source and the source is what is versioned.
- `mesh.unnamed_node` stays an error, not a warning. A warning was tried in February and the unnamed meshes ended up in a playtest build with sockets attached to nothing.
- The 19 rare codes are not worth better messages until one of them passes 30 occurrences in a quarter.
