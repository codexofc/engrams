---
name: a11y-keyboard-drag-drop-dispatch
description: The dispatch board's drag and drop has a full keyboard path (select with Space, move with arrows, drop with Enter, cancel with Escape) and live-region announcements, audited in HF-1360 against WCAG 2.2 AA
type: project
status: active
verified: 2026-04-11
---

# Accessibility of the dispatch board (HF-1360)

Trigger: a public tender for a regional logistics platform in early 2026 required a WCAG 2.2 AA statement. The board, the core of the product, was drag-and-drop only. This note is what was done, what was measured and what remains.

## Keyboard path for drag and drop

The board uses `@dnd-kit` with its keyboard sensor, customised in `src/board/dnd/keyboardCoordinates.ts`:

- `Tab` reaches a load card. `Space` picks it up: the card gets `aria-grabbed="true"` (still announced by current screen readers even though deprecated) and a visible outline, the live region says "Chargement L-2026-004512 saisi. Utilisez les flèches pour choisir un transporteur."

- `ArrowLeft` / `ArrowRight` move the ghost between carrier columns, `ArrowUp` / `ArrowDown` between positions in a column. Each move announces the target: "Colonne Transports Morel, position 3 sur 7."

- `Enter` drops, `Escape` cancels and returns focus to the card in its original place.

- The drop target is also selectable from a menu on the card (`Shift+F10` or the kebab button) listing carriers, for people who find arrow navigation across 30 columns tedious. This menu is what most keyboard users end up using, according to the two testers.

Focus management is the hard part: after a drop, the card has moved in the DOM and React re-renders the column. `useRestoreFocus()` stores the load id before the mutation and focuses `[data-load-id="..."]` after the query cache updates, with a `requestAnimationFrame` retry up to 5 frames because the row virtualiser (see [[table-virtualization-loads-list]]) may not have mounted it yet.

## Announcements

A single `aria-live="polite"` region at the app root (`<LiveAnnouncer>`), written through `announce(message)` from a Zustand store. Rules:

- One message per user action, never per data update. WebSocket updates do not announce (a board with 50 updates a minute would be unusable).

- Messages are cleared after 5 s so the same message can be announced again.

- `assertive` is used for exactly two things: a failed mutation and a load reassigned away from the dispatcher's selection.

## Contrast and focus

- Status colours on cards had 2.8:1 contrast for `BIDDING` (yellow on white). All status colours were redone in [[design-tokens-and-theming]] to reach 4.5:1 for text and 3:1 for the status border, with an icon per status so colour is never the only signal.

- Focus ring: 2 px solid plus 2 px offset, `--hf-focus` token, visible on every interactive element. The reset that removed outlines in 2023 was deleted.

- Dense mode (see [[dispatchers-want-dense-ui]]) keeps the same targets, only reduces padding.

## Audit results

Automated (axe-core through Playwright, see [[e2e-playwright-conventions]]): 0 violations at serious or critical level on the 12 main routes, down from 31. Remaining moderate ones are inside the third-party map control, tracked upstream.

Manual: two external testers (one screen reader user, one keyboard-only user) spent a day each. Tasks: assign a load, filter the board, read a load's timeline, answer a carrier message. All completed. Time to assign a load by keyboard: 25 s against 4 s with a mouse, judged acceptable by the testers because the menu path exists.

## What is not done

- The map is not keyboard operable beyond zoom and a list alternative. A "list of trucks" panel mirrors the map markers, which is the accepted alternative.

- Charts on the reporting pages have `aria-label` summaries but no data table fallback yet. HF-1365, planned.

- Colour-blind simulation was done for deuteranopia only.

## How to keep it

The axe check in CI blocks a merge on serious or critical violations. Every new interactive component needs a keyboard story in the component playground with the key sequence documented. And the two testers are on a small retainer, one half-day per quarter.

## Screen reader specifics found during the audit

Three behaviours that are not in any checklist and cost a day each:

- The Windows screen reader used by the first tester reads `aria-grabbed` but not the live region while a drag is in progress, because its focus mode switches when the card gets `aria-pressed`. Removing `aria-pressed` from the card (it was redundant with the outline) fixed it. The macOS one had the opposite behaviour and read both, twice. The final markup uses `aria-grabbed` only and a `aria-roledescription="chargement déplaçable"` on the card so the role announcement makes sense.

- Virtualised rows (see [[table-virtualization-loads-list]]) that are not in the DOM do not exist for the screen reader, so "row 412 of 2 000" is announced through `aria-rowcount` and `aria-rowindex`, but jumping to row 900 with the screen reader's table navigation lands nowhere. The `scrollToIndex` on keyboard navigation handles arrow keys, not the reader's own table commands. Documented as a limitation with the quick filter as the workaround, accepted by the tester.

- The map's `canvas` announces nothing, which is correct, but focus could land on it and get stuck because the map library traps `Tab` for its own controls. `tabindex="-1"` on the canvas and a skip link "Aller à la liste des camions" before it.

## Statement, review cadence and what triggers a re-audit

The accessibility statement is at `app.halden.example/accessibilite`, in French and English, listing the conformance level (partial, AA on the audited routes), the known limitations (map, charts), the date of the last audit and a contact. It is regenerated from `docs/a11y-statement.md` on each release, and the release checklist has a line to update the date if an audit happened.

Re-audit triggers: a new interactive pattern (anything with drag, keyboard trapping, or custom focus management), a change to the status colours or the focus token, and the yearly full pass. Small changes are covered by the axe run in CI and by the reviewer's keyboard walk-through, which is a required review item for any MR touching `src/board/` or `src/forms/fields/`.

Numbers from the tender that started this: the customer's evaluation gave the board 42 of 50 points on the accessibility grid, against 18 before the work. We did not win the tender, for pricing reasons, and the work stays.
