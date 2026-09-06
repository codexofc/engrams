---
name: event-envelope-format
description: Every torrent message carries seven headers (event-id, event-type, schema-id, produced-at, producer, trace-id, key-version) and a JSON body whose top level is the event itself, no wrapper object, tombstones are null values, keys are JSON
type: reference
status: active
verified: 2026-05-08
---

# The message envelope

## Headers

Every message on every topic has these headers, set by the client wrappers or by the outbox relay ([[producer-acks-and-durability]]); the CI's sampling check refuses a topic where a sampled message lacks any of them.

| Header | Example | Set by | Purpose |
|---|---|---|---|
| `event-id` | `01J0X4...` (ULID) | producer | idempotency reference across systems, unique per message |
| `event-type` | `domain.load.assigned` | producer | equals the topic for `domain.*`, the operation for `cdc.*` (`cdc.app.loads.u`) |
| `schema-id` | `4471` | producer wrapper | registry id of the exact schema version the body validates against ([[schema-registry-compatibility-rules]]) |
| `produced-at` | `2026-05-08T07:14:03.221+00:00` | relay or producer | wall clock at produce, distinct from the broker timestamp and from the business time inside the body |
| `producer` | `api@7.42.1` | producer wrapper | application name and version, matches the TLS certificate CN |
| `trace-id` | `4bf92f3577b34da6a3ce929d0e0e4736` | producer | propagated from the HTTP request that caused the event, so a trace in Tempo spans the API call, the outbox relay and the consumer |
| `key-version` | `1` | producer | how the key is serialised, in case it ever changes |

Header values are UTF-8 strings. Consumers read `event-id` for deduplication when their sink cannot key on the business id, `schema-id` for validation, `trace-id` for logging, and mostly ignore the rest.

## Key

JSON, always, even for a single integer: `{"id": 99120033}` for CDC (the primary key columns by name), `{"load_id": 99120033}` for domain events keyed by aggregate, `{"driver_id": "d_4410"}` for driver streams. The partitioner hashes the bytes, so the key must be serialised canonically (sorted keys, no whitespace), which the wrappers do and the CI's sampler checks by re-serialising.

A key that is a plain string or integer without the JSON object was the 2024 convention and was migrated in 2025 by dual-writing during the [[partition-count-decisions]] changes for `driver.positions`, since the mapping changed anyway.

## Body

JSON, the event itself at the top level. No `{"metadata": ..., "payload": ...}` wrapper: the metadata is in the headers, and a wrapper means every consumer unwraps and every schema has the same two top-level keys. The body's schema is the subject's schema in the registry.

For `cdc.*` the body has the fixed shape from [[cdc-tap-postgres-connector]] (`op`, `table`, `lsn`, `tx_id`, `committed_at`, `before`, `after`). For `domain.*` it is the event's fields: a `domain.load.assigned` body is

```
{
  "load_id": 99120033,
  "carrier_id": 8812,
  "driver_id": "d_4410",
  "assigned_by_user_id": 55120,
  "assigned_at": "2026-05-08T07:14:02+00:00",
  "pickup_window_start": "2026-05-09T06:00:00+02:00",
  "pickup_window_end": "2026-05-09T10:00:00+02:00",
  "version": 3
}
```

`version` is the aggregate version after the change, the guard consumers use for out-of-order handling ([[exactly-once-vs-idempotent-consumers]]). Every `domain.*` schema has it.

Business timestamps inside the body are RFC 3339 with offset, in the timezone that makes sense to the business (a pickup window in the site's local time), field names ending in `_at` for instants and `_start`/`_end` for windows. `produced-at` in the header is UTC. The two are different things and the March 2026 invoice replay ([[replay-2026-03-invoice-projection]]) is what happens when a consumer confuses them.

## Tombstones

A message with a null body and a key is a tombstone, meaning "this key no longer exists", used on compacted `cdc.*` topics after a delete. Headers are still set (`event-type: cdc.app.loads.d`). Consumers must handle a null body on any compacted topic; the wrappers surface it as an explicit `Tombstone(key)` variant rather than a null that surprises a deserializer.

## Size

Bodies are 200 bytes (`driver.positions`) to 40 KB (`cdc.app.loads` with `before` and `after` of a wide row). `message.max.bytes` is 1 MB; a body approaching it is a design problem, and the one that did (a `domain.document.available` that embedded the OCR result, 2025) was fixed by putting a reference to the object store in the event instead. Compression is `zstd` at the producer, which turns JSON into a sixth of its size on the wire and on disk.

## What is not in the envelope

- No `tenant` or `org_id` header: multi-tenancy is in the body where the schema can type it, and no consumer partitions by tenant.

- No `retry-count`: retries are the consumer's business ([[dead-letter-topics-convention]] has the `dlq-attempt` header for the one place it matters).

- No signature or encryption at the message level: the cluster is inside the datacentre network with mutual TLS per client and ACLs per topic, and the one topic with sensitive columns filters them at the source.
