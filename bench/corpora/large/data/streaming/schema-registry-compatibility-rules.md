---
name: schema-registry-compatibility-rules
description: Schemas are JSON Schema documents in the torrent registry at registry.hf.internal, one subject per topic, BACKWARD compatibility enforced by the CI, no field removal without a deprecation cycle, unit in the field name or in x-unit
type: reference
status: active
verified: 2026-05-08
---

# Schemas and compatibility

## The registry

`registry.hf.internal` is the schema registry that ships with torrent, one subject per topic (subject name equals topic name, see [[topic-naming-and-ownership]]), JSON Schema draft 2020-12 as the schema language. Avro was the default choice of every tutorial; we chose JSON Schema because the producers are PHP, TypeScript and Rust and the consumers are the same plus SQL people reading `raw.*` tables in the warehouse, and a JSON payload everyone can read in `torrent-console-consumer` beats a compact binary that needs the registry to be legible. The size cost is real (a `driver.positions` message is 310 bytes in JSON, would be about 90 in Avro) and accepted.

Schemas live in Git (`data-platform/schemas/<subject>/<version>.json`) and are pushed to the registry by the CI. A producer that registers a schema at runtime is refused by the registry's ACL (`register` is CI only). The message header `schema-id` carries the registry id ([[event-envelope-format]]).

## Compatibility mode

`BACKWARD` on every subject, checked by the registry at registration and by the CI before that with the same library. `BACKWARD` means a consumer on the new schema can read messages produced with the old one, which is what a replay needs ([[replay-procedure-runbook]]): the consumer is always newer than the oldest message it may read.

Concretely allowed:

- adding an optional field (with `default` or nullable);

- widening an enum (adding a value), provided consumers treat unknown values as "other" (this is a convention, the schema cannot enforce it, the review does);

- relaxing a constraint (a `maxLength` going up).

Not allowed:

- removing a field, renaming a field (which is a removal plus an addition);

- changing a type (`integer` to `string`), including changing a unit without changing the name, which is the February 2026 ETA incident on the ML side and the reason for the unit rule below;

- making an optional field required;

- narrowing an enum.

## Deprecation cycle for removals

1. Mark the field `"deprecated": true` with `x-deprecated-since: <date>` and `x-removal-after: <date>` at least 90 days out. The CI lists deprecated fields in the weekly schema report.

2. The producer keeps writing it. Consumers stop reading it.

3. After the date, `x-removal-after` past, the consumer list from `topics.yaml` is checked, each consumer's owner confirms in the MR, and the field is removed with `FULL` compatibility temporarily disabled for that one registration (the CI needs a `--allow-incompatible` flag that requires two approvals from the data team).

Done four times since 2025. Median time from deprecation to removal: 140 days.

## Units

A numeric field carries its unit either in the name (`distance_km`, `duration_minutes`, `amount_cents`) or in an `x-unit` annotation when the name would be silly. The CI refuses a numeric field with neither. Money is always integer cents with a sibling `currency` field, never a float. Timestamps are RFC 3339 strings with offset, field names ending in `_at`. Durations are integers with the unit in the name. This rule came from the ETA drift incident on the ML side (minutes became seconds under the same name) and is the one rule in this note that has prevented a repeat we know of: a March 2026 MR changed `remaining_range_km` to metres and the CI asked why the name still said km.

## Validation at produce time

Producers validate against the schema before sending (`SchemaValidatingProducer` in the PHP and TypeScript client wrappers, `torrent_client::validate` in Rust). The broker does not validate; the registry is a lookup, not a gate. A message that fails validation is not sent and is counted (`torrent.produce_rejected{topic}`), and the producer decides whether to fail the transaction or to log. The API fails the transaction: a `domain.*` event that cannot be produced means the business operation did not happen, by design.

Consumers validate too, on the schema id in the header, and route failures to the dead letter topic ([[dead-letter-topics-convention]]). The [[incident-2026-02-poison-message-bids]] is what happens when a consumer skips that step.

## Numbers

- 218 subjects, 640 versions, the most versioned is `cdc.app.loads` (31 versions, one per table migration touching the table).

- Weekly schema report: deprecated fields pending removal (currently 6), subjects without a producer in 30 days (2, both `ops.*` from decommissioned jobs, to be deleted), subjects whose latest version is not what the producer writes (0, checked by sampling 100 messages per topic against the registered version).

## What we do not do

- No `FORWARD` or `FULL` mode: consumers are deployed before producers as a habit, and `BACKWARD` matches that.

- No schema inference from payloads. A schema is written by a person and reviewed by the topic owner and one data person.

- No shared "common types" library across subjects beyond the envelope: it seemed elegant and it made every subject depend on every other subject's release. Fields are duplicated per subject, the CI checks that a field with the same name has the same type across subjects (a warning, not a failure).
