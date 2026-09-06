---
name: design-tokens-and-theming
description: Design tokens are CSS custom properties generated from tokens.json (colour, spacing, radius, type scale), light and dark themes via data-theme on html, status colours are semantic tokens with an icon companion since the a11y audit
type: reference
status: active
verified: 2026-04-16
---

# Design tokens and theming

## Source of truth

`design/tokens.json`, edited by design and front together, one merge request per change. A build step (`pnpm tokens:build`, Style Dictionary) generates:

- `src/styles/tokens.css` with CSS custom properties on `:root` and `[data-theme="dark"]`

- `src/styles/tokens.ts` with typed constants for the rare cases where a token is needed in JavaScript (the map marker colours, chart palettes)

Nobody edits the generated files. A CI check regenerates and diffs.

## Naming

`--hf-<category>-<role>[-<state>]`:

- Colour: `--hf-color-bg`, `--hf-color-bg-raised`, `--hf-color-text`, `--hf-color-text-muted`, `--hf-color-border`, `--hf-color-accent`, `--hf-color-accent-hover`, `--hf-color-danger`.

- Status (semantic, one per load status): `--hf-status-open`, `--hf-status-bidding`, `--hf-status-dispatched`, `--hf-status-in-transit`, `--hf-status-delivered`, `--hf-status-cancelled`, each with `-bg` and `-fg` variants.

- Spacing: `--hf-space-1` (4 px) to `--hf-space-8` (48 px), on a 4 px grid.

- Radius: `--hf-radius-sm` (4 px), `-md` (8 px), `-lg` (12 px).

- Type: `--hf-font-size-1` (12 px) to `-6` (24 px), `--hf-font-sans`, `--hf-font-mono`.

- Focus: `--hf-focus` (colour) and `--hf-focus-ring` (the full `box-shadow` value).

Components use tokens only. A raw hex value in `src/` fails `stylelint` (`hf/no-raw-color`).

## Themes

`<html data-theme="light|dark">`, set from the user's preference in `useShellStore`, defaulting to `prefers-color-scheme`. The dark theme overrides the colour tokens only. Every component is built and reviewed in both, the component playground has a theme toggle in the toolbar.

Dense mode (see [[dispatchers-want-dense-ui]]) is not a theme, it is `data-density="dense"` on the board container and it overrides the spacing tokens locally.

## Status colours after the accessibility audit

The 2025 palette failed contrast on `bidding` (2.8:1) and `open` (3.9:1) for text. Redone in HF-1360 (see [[a11y-keyboard-drag-drop-dispatch]]):

| Status | fg on bg contrast | Icon |
|---|---|---|
| open | 7.1:1 | circle outline |
| bidding | 5.2:1 | gavel |
| dispatched | 6.4:1 | truck outline |
| in transit | 6.8:1 | truck filled |
| delivered | 7.5:1 | check |
| cancelled | 8.0:1 | cross |

The icon is mandatory next to the colour wherever the status is shown, `StatusBadge` is the only component allowed to render a status, and it always renders both.

## Charts

Chart colours are a separate ordered palette `--hf-chart-1` to `--hf-chart-8`, chosen to be distinguishable under deuteranopia simulation and to keep 3:1 against both backgrounds. Series beyond 8 are not allowed, the chart component groups the rest as "Autres".

## What is not a token

Component-specific dimensions (the 44 px row height of the virtualised table, the 56 px board column header) are constants in the component. Making them tokens invited people to change them from CSS overrides, which broke the virtualiser once.
