---
name: feature-flags-web
description: Front feature flags come from GET /internal/web/config at login and every 5 min, read through useFlag('name'), targeted by organisation and user role, with a local override panel in non-production builds
type: reference
status: active
verified: 2026-04-22
---

# Feature flags on the front

Same server-side table and admin UI as the mobile app (`sys_feature_flags`), separate endpoint `GET /internal/web/config` because the targeting dimensions differ (organisation and role here, device and app version there).

## Reading

`useFlag('board.bulk_assign')` returns a boolean, `useConfigValue('board.page_size', 50)` returns a typed value with a default. Both read from `useShellStore.flags`, populated at login and refreshed every 5 minutes by a background query, and on every WebSocket `config.changed` event so a flag flip reaches open tabs within seconds.

The flag names are declared in `src/flags.ts` as a `const` object with a comment per flag holding the ticket and the intended removal date. A flag not declared there is a type error. This is how we noticed 9 flags that were on for everyone for months in 2025 and removed them.

## Targeting

Rules evaluated server-side, first match wins: organisation id, user role (`dispatcher`, `shipper_admin`, `support`), percentage on user id, default. No per-browser targeting. The support role sees every flag on by default, so support can see what a customer will see next week, which they asked for.

## Local override

In `dev` and `staging` builds, `Ctrl+Shift+F` opens a panel listing every declared flag with its current value and a toggle. Overrides live in `localStorage` under `hf.flags.override` and beat the server value. The panel is compiled out of production builds (`import.meta.env.PROD` check, tree-shaken).

## Rules

- A flag guards behaviour, never text or colour. Those are config values or tokens.

- The default value in `src/flags.ts` is the safe one (usually off). The server default can differ.

- Removal: when a flag is 100 % on for a month, the flag and both branches of the `if` are removed in one PR. The declared removal date is checked by a lint that warns 2 weeks before and fails after.

- No flag inside a render loop of the virtualised table without memoising the read, it re-rendered every row on every flags refresh once. `useFlag` is memoised now, but do not read flags in `renderRow`.

## Interaction with the API's feature header

Unrelated mechanisms. `X-Halden-Features` (see [[api-client-generation-openapi]]) tells the API which response shapes the client understands; it is a list in code, not a flag. A feature flag decides whether the UI shows something. A new feature often needs both: the header entry ships with the code, the flag turns the UI on later.

## Related

State placement in [[dispatch-board-state-zustand]]. The mobile equivalent is documented in the mobile project.
