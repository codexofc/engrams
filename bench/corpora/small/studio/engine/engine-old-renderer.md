---
name: engine-old-renderer
description: The 2024 hard-wired Givre renderer with manual barriers, retired in February 2026 for the frame graph
type: reference
status: archived
superseded_by: [[renderer-frame-graph]]
verified: 2025-10-01
---

Until February 2026 the Givre renderer was a fixed sequence of passes called one after another from `Renderer::render()`, with barriers placed by hand between them and every intermediate texture allocated once at startup at the maximum resolution.

Why it had to go:

- Every new effect meant editing the sequence and re-checking every barrier by hand. Two GPU hangs in 2025 came from a missing barrier after the volumetric pass was added.
- No aliasing: 14 intermediate textures always resident, 480 MB of video memory at 1440p, which pushed the low-end target over its budget.
- Disabling a pass (for a platform or a quality setting) was a chain of `if` in the sequence, and the debug pass was shipped in a release build once because of it.

The frame graph in [[renderer-frame-graph]] replaced it. The port took six weeks and the render output was compared pixel by pixel on the 40 reference captures during the switch; 3 differed by one bit in the bloom, accepted.
