---
name: studio-glossary
description: The words Brume Studio uses in identifiers and tickets: Givre, Marée, cutter, harbour, chunk, lane, and the names not to use
type: reference
status: active
verified: 2026-01-10
---

The identifiers, tickets and notes use these words. In French prose the French word is fine; in identifiers, file names and event names, the English column is the only one.

| English (identifiers) | French (prose) | Meaning |
|---|---|---|
| Givre | Givre | the in-house Rust engine, crates `givre-*` |
| Marée | Marée | the game in production, crate `maree` |
| cutter | cotre | the player's boat; never `boat` in identifiers, there are other boats |
| harbour | port | a dockable place; `harbour_master` for the character |
| bay | baie | a region of the map, 4 in the game |
| chunk | chunk | a 64 m world cell, unit of streaming |
| pack | paquet | the pipeline's output file per chunk and platform |
| lane | lane | one CI job sequence (code, assets, playtest) |
| milestone | jalon | a 6-week cycle, tickets `BR-M<nn>` |
| playtest | playtest | a test session with external testers |
| replay | replay | recorded inputs plus seed for deterministic reproduction |
| socket | socket | a named attachment point on a mesh |
| overlay | overlay | the `F3` in-game debug display |
| low-end target | cible basse | the portable console build, never named by brand in notes |
| reference machine | machine de référence | the desktop used for measurements |

## Names not to use

- `boat` for the player's vessel in code (use `cutter`); `boat` is the generic floating object type.
- `level` for a chunk or a bay; `level` is reserved for the editor's file (`.lvl`) which contains one bay.
- `build` alone; say code build, asset build or playtest build.

Naming of asset files, which reuses these words, is in [[naming-assets]].
