---
name: webhook-incident-2026-06-payload-v3-null-driver
description: June 2026, v3 payloads sent driver as {} instead of null and 14 integrators broke for two days; fixtures now hand-written, 30-day notice rule
type: project
status: active
verified: 2026-07-02
---

# Incident 2026-06-17: v3 payloads and the null driver (HF-3150)

Not an outage. Every delivery went out, every endpoint answered. Fourteen integrators on payload v3 rejected or mis-processed `load.dispatched` events for two days, and we learned it from their tickets.

## What changed

A release on 2026-06-16 modified the v3 payload builder so that `driver` on `load.*` events became `{ "id": ..., "name": ..., "phone_last4": ... }` instead of the flat `driver_id` and `driver_name` of v2. Intended and announced for v3. What was not intended: when no driver is assigned yet (always the case on `load.dispatched`, the carrier assigns after accepting), the new builder emitted `"driver": {}` instead of `"driver": null`, because the serializer's null-skipping context dropped the null fields inside the object and left the shell.

## Effects

- 9 integrators had code like `if payload.driver: notify(payload.driver.name)`. An empty object is truthy in JavaScript and in PHP arrays. They tried to read `name` of nothing and either crashed (500, so our retries hammered them for two days, see [[webhook-retry-schedule-v2]]) or created a driver record with an empty name in their TMS.

- 3 integrators with strict schema validation (a JSON schema with `driver: object|null` and `required: [id]` inside the object) rejected with 422 and logged it. Those were the ones who told us, on the 17th.

- 2 did nothing wrong and nothing happened; their code checked `driver?.id`.

The contract test `WebhookPayloadContractTest` compares payloads to frozen examples per version, but the frozen v3 example for `load.dispatched` had been regenerated on the 16th from the new builder, so it was frozen wrong. The test compared the builder to itself.

## Fix

- 2026-06-18: hotfix, `driver` is `null` when absent, in v2 and v3. The serializer context for nested value objects now uses an explicit `WebhookNullPolicy` that keeps `null` on declared nullable fields and never emits an empty object for a nullable one.

- Frozen examples are now written by hand in `tests/Webhook/fixtures/v3/*.json` and reviewed; the test fails if the builder differs, and a script `bin/console webhook:fixtures:diff` shows what changed so the reviewer sees it. Regenerating fixtures from the builder is no longer a command.

- Integrators who received `{}` for two days: we did not replay ([[webhook-replay-tool]] would have re-sent the frozen wrong payload, a replay does not re-render). We sent each of the 14 a list of affected `delivery_id`s and `load_id`s so they could fetch current state.

## Process changes

- Any payload shape change, even inside a version that is "still in beta" as v3 was, goes to the integrator changelog 30 days before, with a sample payload before and after. v3 stopped being "beta" that week because 40 integrators were on it in production whatever we called it.

- The [[webhook-payload-versioning-v2-v3]] note now states the nullability rule: a nullable field is `null`, never absent and never an empty container.

- The contract test suite grew a "nullability" case per event type, generated from the event's schema, that builds the payload with every optional relation absent and asserts the field is literally `null`.

## What we learned

- A frozen example that is regenerated automatically is not frozen.

- "Beta" is a label we give ourselves. Integrators ship what works.

- The integrators who found it in an hour were the ones validating against a schema. The guide now recommends it and publishes our JSON schemas per version.

## Figures

14 affected out of 41 on v3. 3 600 deliveries with `{}`. 2 days. 0 lost events. About 60 support hours including the individual e-mails, which is more than the fix took.
