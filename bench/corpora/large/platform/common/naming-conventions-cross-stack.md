---
name: naming-conventions-cross-stack
description: Cross-stack naming, snake_case in JSON and SQL, camelCase in TypeScript and Dart, PascalCase types, kebab-case for routes, env vars prefixed HF_ for app config and APP_ for Symfony, business terms fixed by the glossary
type: reference
status: active
verified: 2026-01-28
---

# Naming across the stack

The point is that a field called `pickup_window_start` in the database is `pickup_window_start` in JSON, `pickupWindowStart` in TypeScript and Dart, and nothing else, so a search finds all of it.

## Casing

| Where | Case | Example |
|---|---|---|
| SQL tables and columns | snake_case | `load_events.occurred_at` |
| JSON payloads (API, webhooks, sync) | snake_case | `"pickup_window_start"` |
| PHP properties and methods | camelCase | `$load->pickupWindowStart` |
| TypeScript and Dart identifiers | camelCase | `load.pickupWindowStart` |
| Types, classes, enums | PascalCase | `LoadStatus`, `CarrierDocument` |

## Casing (continued)

| Where | Case | Example |
|---|---|---|
| Enum values in JSON and SQL | UPPER_SNAKE | `IN_TRANSIT` |
| URL paths | kebab-case, plural nouns | `/v2/carrier-documents` |
| Query parameters | snake_case | `?pickup_after=` |
| HTTP headers | `X-Halden-Kebab-Case` | `X-Halden-Features` |
| Kubernetes objects | kebab-case | `halden-api-worker-exports` |
| Metrics | snake_case with unit suffix | `hf_sync_pull_duration_seconds` |

Conversion between snake_case JSON and camelCase code is done by the serializer (Symfony name converter, the generated TypeScript types keep snake_case and a mapping layer camelises at the API client boundary, `json_serializable` with `fieldRename: snake` in Dart). Never by hand.

## Environment variables

- `APP_*` is reserved for Symfony framework settings (`APP_ENV`, `APP_SECRET`, `APP_DEBUG`).

- `HF_*` for our own configuration in every component: `HF_API_SPEC_URL`, `HF_RATE_LIMIT_ENABLED`, `HF_TILES_URL`.

- `DATABASE_URL`, `DATABASE_URL_MIGRATIONS`, `DATABASE_URL_LISTEN`, `MESSENGER_TRANSPORT_DSN`, `REDIS_URL` keep their conventional names.

- Secrets are never in variables named `*_KEY` alone, the name says what it is for: `HF_WEBHOOK_SIGNING_KEY`. And they come from the secret store, see the ops notes.

## Business terms (glossary)

Fixed in `docs/glossary.md`, in French and English, and the code uses the English term:

- **Load** (chargement), not shipment, not order, not freight.

- **Bid** (offre), not quote, not proposal.

- **Carrier** (transporteur), **Shipper** (chargeur), **Driver** (chauffeur), **Dispatcher** (dispatcher, the same word in French).

- **Stop** (arrêt) for a pickup or delivery point, **Window** (créneau) for its time range.

- **POD** (preuve de livraison), always the acronym in code, `ProofOfDelivery` nowhere.

- **Lane** (ligne) for an origin-destination pair.

A new term goes in the glossary before it goes in a class name. The word "order" is banned because it means three things to three teams.

## Tickets and branches

`HF-<number>` everywhere: branch names, commit subjects, `TODO(HF-1234)`, flag declarations, changelog lines. See [[branching-and-pr-flow]] and [[commit-message-convention]].
