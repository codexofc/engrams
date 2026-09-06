---
name: level-editor-undo
description: Level editor undo as a command stack with inverses grouped by gesture, 200 deep, global since the cross-chunk duplicate bug
type: project
status: active
verified: 2026-03-26
---

## Design

Every edit in the level editor is a command implementing `EditCommand { apply, invert }`. The editor keeps a stack of 200 commands; undo applies the inverse, redo re-applies. Commands produced by one gesture (a drag that moves 40 objects, a paint stroke) are grouped into one `Composite` so that one undo reverts the whole gesture.

Commands act on the live world through the same path as the hot reload ([[editor-hot-reload]]); the level file is rewritten on save from the world state, not from the command log.

## Cross-chunk fix (BR-388)

Moving an object from chunk A to chunk B was two commands (remove from A, add to B) in two different chunk-local stacks, because stacks were per chunk to keep them small. Undo in chunk B re-added the object in B without removing it from A: a duplicated object, which shipped in the February playtest as a floating buoy.

Since March there is one global stack, and a command records the chunks it touches; saving a chunk marks the commands touching it as `saved`, and undo past a saved command warns ("this will change chunk A which was saved") rather than refusing.

## Not undoable

- Asset imports and reimports: they change files on disk and the pipeline cache; undoing them would mean restoring the previous file, which version control already does. The editor says so in the import panel.
- Play mode changes: entering play mode snapshots the world and leaving it restores the snapshot, which is a different mechanism from undo. Edits made during play mode are lost, by design, and the editor shows a red border in play mode since two designers lost work in January.

## Numbers

Undo of a 40-object drag: 0.3 ms. The stack of 200 composites holds at most a few MB. A session of 8 hours produced 6 400 commands on the busiest day measured; the 200-entry limit has not been raised because nobody asked.
