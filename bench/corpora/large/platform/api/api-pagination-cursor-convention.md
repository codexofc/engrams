---
name: api-pagination-cursor-convention
description: List endpoints paginate with an opaque base64 cursor (after=, limit<=200) sorted on (created_at, id), offset pagination is gone since HF-1402
type: reference
status: active
verified: 2026-04-02
---

# Cursor pagination on list endpoints

Every collection endpoint (`GET /v2/loads`, `/v2/bids`, `/v2/invoices`, `/v2/carriers/{id}/documents`) paginates the same way since HF-1402 shipped in January 2026. Offset pagination is described in [[api-pagination-offset-convention]] for history only.

## Contract

Query parameters:

- `limit`: 1 to 200, default 50. Above 200 you get a 422 with code `limit_too_large`.
- `after`: opaque cursor from the previous response. Clients must not parse it.
- `sort` is not accepted anymore on cursor-paginated endpoints. Order is fixed per endpoint and documented in the OpenAPI description, usually `created_at DESC, id DESC`.

Response:

```json
{
  "items": [ ... ],
  "next_cursor": "eyJjIjoiMjAyNi0wMy0xMlQwOToxNDoyMloiLCJpIjoiOGE0ZjJjMTctYjNhZC00YzE5LWEyOWMtZjIwZDE4Yjc4YmE1In0",
  "has_more": true
}
```

`next_cursor` is `null` when `has_more` is false. There is no `total`. Product wanted a total for the dispatch board; we refused because `COUNT(*)` over `loads` with the usual filters took 380 ms at p95 on prod in December 2025 (about 4.1 M rows). The board shows "50+" instead, see the web note on the loads table.

## Implementation

`App\Api\Pagination\CursorPaginator` takes a `QueryBuilder`, the sort tuple and the decoded cursor. It adds `WHERE (created_at, id) < (:c, :i)` as a row comparison so PostgreSQL can use the composite index. Each paginated entity has an index named `idx_<table>_cursor` on `(created_at DESC, id DESC)`. `CursorIndexTest` reads `doctrine:schema:validate` output and checks that every entity tagged with `#[CursorPaginated]` has it.

The cursor is `base64url(json_encode(['c' => created_at ISO8601 with microseconds, 'i' => uuid]))`. Microseconds matter: we lost rows on prod for two days because the first version truncated to seconds and two loads created in the same second at a page boundary were skipped (HF-1433).

Cursor decoding failures return 422 `invalid_cursor`, never 400, so the mobile app can distinguish them from malformed JSON.

## Why not keyset on `id` only

The UUIDs are v4, random, so ordering on id alone gives a meaningless order. UUID v7 was discussed for new tables (see [[entity-naming-and-table-prefixes]]) but the existing 40 tables are v4 and we are not rewriting primary keys.
