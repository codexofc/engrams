---
name: tools-team-preferences
description: Tools team preferences: every error says what to do, no silent asset fixes, the editor is the game binary, no manuals
type: user
status: active
verified: 2026-02-20
---

Preferences of the tools team (two people), applied to `brumepipe`, the editor and the CI.

- **Every error says what to do.** Code, one-line fix, path, link to the convention. The count of failures per code is reviewed monthly ([[asset-import-failures-2026]]).
- **No silent fix of a source asset.** The pipeline refuses, it does not resize, rename or convert. The source is what is versioned and what the artist sees.
- **The editor is the game binary** (`maree --editor`). No separate editor application, no separate rendering path. What the designer sees is what ships, and hot reload ([[editor-hot-reload]]) works because of it.
- **A tool that needs a manual is not finished.** The panel explains itself or it goes back. Written docs are for conventions, not for buttons.
- **Speed is measured on an artist's machine**, not on the CI runners. The reference is the oldest workstation in the art room.
- **CI lanes have a duration alert**, not only a result ([[ci-cache-misses-lesson]]).
- **French for artist-facing text**, English for identifiers and logs. Error messages shown in the editor are in French; the same message in the CI log is in English, from the same code.
- **Tools requests go through a ticket with a screen recording** of the problem. A request without a recording waits until there is one; it takes the artist 30 seconds and saves an hour of guessing.
