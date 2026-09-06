---
name: inference-service-latency
description: ml-infer serves ETA at 6 ms p50 and 21 ms p99 within a 50 ms budget, and the March 2026 connection pool regression to 140 ms p99
type: project
status: active
verified: 2026-04-08
---

## Service

`ml-infer` (Rust) serves `POST /v1/eta`, `POST /v1/rest-detect` and `POST /v1/batch/eta`. Two instances behind the internal load balancer, 2 vCPU each, 25 rps average, 180 rps at the 08:00 peak when dispatch recomputes every in-progress load. Callers set a 50 ms timeout and fall back to the routing engine's time plus a margin, with a `fallback` flag in their response so the dispatch UI shows a dashed ETA.

## Where the time goes (ETA, April 2026)

| Step | p50 | p99 |
|---|---|---|
| request parsing and validation | 0.1 ms | 0.3 ms |
| feature lookups (4 keys in parallel) | 1.8 ms | 9 ms |
| feature vector assembly, range checks | 0.2 ms | 0.5 ms |
| six booster evaluations | 2.1 ms | 4 ms |
| response | 0.1 ms | 0.2 ms |
| total | 6 ms | 21 ms |

The boosters are evaluated sequentially; parallelising them was tried and the thread handoff cost more than it saved at this tree count (600 trees of depth 8 each).

## The March 2026 regression

- 2026-03-04: p99 goes from 21 ms to 140 ms, p50 unchanged. Fallback rate at the callers from 0.1 % to 4 %.
- The feature store's online lookup ([[feature-store-design]]) had been moved to a new key-value cluster the day before by the platform team. `ml-infer` used a connection pool of 8 per instance, sized for the old cluster's per-connection multiplexing; the new one does not multiplex, so at 180 rps with 4 lookups each, 720 lookups per second waited on 16 connections.
- Fix on 2026-03-05: pool of 64 per instance, and lookups for the same request pipelined on one connection. p99 back to 19 ms.
- Added: an alert on `ml_infer.feature_lookup_p99 > 15 ms` separate from the total, and a load test in CI at 300 rps against a staging store, which would have caught the pool sizing.

## Batch endpoint

`POST /v1/batch/eta` takes up to 500 loads and is what dispatch's 10-minute recompute uses (one call per shard of loads instead of 5 000 single calls). Feature lookups are batched by key type (500 load keys in one multi-get), which brings the per-load cost to 0.4 ms. The batch endpoint has its own 2 s timeout.

## Model reload

Hot reload on the `current` pointer change ([[model-registry-conventions]]): the new boosters are loaded in a background thread, validated on the 100 reference loads, then swapped atomically. During the 3 to 4 s of loading, requests continue on the old model. A failed validation logs `model.reload_rejected` and keeps the old one; this happened once when a `features.lock` referenced a renamed feature.

## Not done

GPU or a separate model server. Six boosters at 2 ms do not need it, and the team prefers a service it can read end to end ([[ml-team-preferences]]).
