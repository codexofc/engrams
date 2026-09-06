---
name: position-dedup-rules
description: A position is dropped if (vehicle, second, provider) was seen in 15 min or if within 5 m and 5 s of the last kept one; removes 78 % of raw, 90 % for Trakko
type: project
status: active
verified: 2026-03-19
---

# Position deduplication (HF-2058, revised HF-2126)

Raw input is 41 million positions a day, of which about 9 million are new information. The rest is the same fix seen again (Trakko's sliding window, Geolyx retries, app re-uploads after a lost acknowledgement) or a fix so close to the previous one that keeping it adds storage and nothing else. `DedupStage` in the pipeline ([[position-ingestion-pipeline]]) applies two rules.

## Rule 1: exact duplicate

Key: `(vehicle_id, floor(device_ts to 1 s), provider)`. Stored in a Redis set per vehicle, `dedup:{vehicle_id}`, members are `"{ts_s}:{provider}"`, TTL 15 minutes refreshed on write. Seen before: `outcome=dup_exact`, dropped.

Why a second, not a millisecond: Trakko's history endpoint returns epoch seconds while its live endpoint returns ISO with milliseconds, so the same position arrives with two different sub-second values. Rounding to the second makes them equal. Two genuine fixes from the same device within one second do not happen with any of our sources (minimum interval is 5 s on the app when moving).

Why the provider is in the key: a vehicle with both a Trakko unit and the app produces two streams that are *not* duplicates of each other; [[position-source-priority]] decides which stream is shown, but both are kept.

Why 15 minutes: Trakko's sliding window is 5 minutes, Geolyx retries within 24 h but a retried batch is already caught by `batch_id` at the door, and the app's re-upload window is 10 minutes. Fifteen covers the cases that reach this stage. Redis memory for this: about 2 900 vehicles times 180 members times 20 bytes, under 20 MB.

## Rule 2: near-duplicate

If the position is within **5 metres** (haversine) and **5 seconds** of the last *kept* position of that vehicle (Redis `lastpos:{vehicle_id}`, also used by the plausibility stage), `outcome=dup_near`, dropped. A parked truck reporting every 10 s produces one kept position per 5 s at most; with rule 2 it produces one per 10 s, because 10 s is outside the 5 s. That was the point of the HF-2126 revision: the original rule used 30 m and 60 s and erased the small movements of a truck manoeuvring in a yard, which the geofence consumer needed ([[geofence-arrival-detection]]).

The persistence layer has a belt-and-braces unique index on `(vehicle_id, ts, provider)` per partition ([[positions-table-partitioning]]), so a Redis flush does not create duplicates in the table, only a burst of constraint violations that the `COPY` batch handles by falling back to row-by-row `ON CONFLICT DO NOTHING` for that batch.

## What we deliberately do not deduplicate

- Across providers (see above).

- Across the 15 minute window. If a position from 20 minutes ago arrives again (a Trakko history backfill), it hits the unique index and is dropped there, at higher cost. Acceptable because backfills are rare.

- Positions with different `accuracy_m` for the same key: the first one wins. The app sometimes sends a coarse fix then a refined one within the same second; we lose the refinement. Measured impact on ETA: none visible. Not worth the complexity.

## Numbers (March 2026, one week)

| Provider | Raw | dup_exact | dup_near | Kept |
|---|---|---|---|---|
| Trakko | 15.1 M | 13.2 M (87 %) | 0.5 M (3 %) | 1.4 M |
| Geolyx | 4.8 M | 0.3 M (6 %) | 0.9 M (19 %) | 3.6 M |
| App | 21.3 M | 1.1 M (5 %) | 16.0 M (75 %) | 4.2 M |

The app's high near-duplicate rate is expected: it samples every 10 s while moving and the truck is often stopped at lights, docks and borders. Reducing the sampling rate on the device would save battery, and the mobile team's battery budget note covers why they keep 10 s anyway (distance filter on device is 25 m, which is coarser than our 5 m, so the two filters overlap and ours catches what theirs lets through at low speed).

## Testing

`DedupStageTest` replays a recorded hour of production input (anonymised vehicle ids) and asserts the kept count within 0.5 % of the reference. The recorded hour lives in the test harness fixtures ([[telematics-integration-test-harness]]).
