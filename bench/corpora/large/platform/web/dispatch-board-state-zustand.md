---
name: dispatch-board-state-zustand
description: Dispatch front state is split into server cache (TanStack Query, staleTime 30 s) and UI state (Zustand slices per board), no global store for server data since HF-1150
type: reference
status: active
verified: 2026-03-18
---

# State management on the dispatch front

Two kinds of state, two tools, and the rule that they never mix. Replaces the single Redux store described in [[web-state-redux-legacy]].

## Server state: TanStack Query

Everything that comes from the API lives in the query cache and nowhere else. Query keys are built by `queryKeys` in `src/api/keys.ts` (`['loads', 'list', filters]`, `['loads', 'detail', id]`, `['carriers', 'detail', id]`), never inline strings, so invalidation is greppable.

Defaults in `src/api/queryClient.ts`:

- `staleTime: 30_000`. The board refetches on focus and on WebSocket events (see [[websocket-live-updates]]), 30 s is the fallback.

- `gcTime: 5 * 60_000`.

- `retry: 2` for GET, `0` for mutations. A failed mutation is shown to the user, not retried silently, because a dispatcher who clicked "Assign" twice by accident must not get two assignments.

- `refetchOnWindowFocus: true`, dispatchers switch tabs constantly.

Mutations use `onMutate` optimistic updates only for status changes on the board (drag a load to a carrier column), with rollback from the snapshot. Everything else waits for the server. The optimistic path exists because the drag felt broken with 300 ms of latency.

## UI state: Zustand

One store per board area, in `src/stores/`:

- `useBoardStore`: selected load ids, column collapse state, active filters, sort. Persisted to `localStorage` under `hf.board.v3` with `zustand/middleware` `persist`, version bumped when the shape changes (a migration function maps v2 to v3, which dropped the `groupBy` field).

- `useMapStore`: viewport, layer toggles, followed truck id. Not persisted.

- `useShellStore`: sidebar open, current organisation, feature flags snapshot.

Selectors are always narrow (`useBoardStore(s => s.filters.status)`), never `useBoardStore()` without a selector, because the board re-rendered 40 times per second during a drag before the rule. ESLint rule `hf/no-bare-store-hook` enforces it.

## What goes where: the test

If losing it on refresh would annoy the user but not lose data, it is UI state. If it can be refetched from the API, it is server state. Form drafts are the exception, see [[forms-react-hook-form-zod]]: they live in the form library's state, and the load creation draft is additionally saved to `sessionStorage` every 2 s.

## Derived data

Anything computed from server data (loads grouped by carrier column, counts per status) is computed with `useMemo` from the query data in the component that needs it, or with a `select` option on the query. No derived state stored anywhere. The one exception is the board's `columnOrder`, which is UI state that references carrier ids from server state, and is reconciled in `useColumns()` when carriers change (an unknown carrier id is dropped, a new carrier is appended).
