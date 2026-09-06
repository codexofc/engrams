---
name: rightsizing-2025-q4-requests-limits
description: Q4 2025 pass over requests on hf-main: 158 cores requested for 61 used, cut 38 % on 40 workloads with p95 plus 30 %, 4-node order cancelled, HF-4705
type: project
status: active
verified: 2026-01-20
---

# Rightsizing requests, Q4 2025

## Why requests, not usage

On a bare-metal cluster the money is spent when a node is bought. What decides whether a node is bought is whether the scheduler can place pods, and the scheduler places by requests. A Deployment requesting 4 cores and using 0.6 occupies 4 cores of the cluster's schedulable capacity, forever, whatever it does. In October 2025 the sum of CPU requests on `hf-main` was 158 cores on 22 worker nodes of 32 vCPU (704 available, 22 % requested) while the p95 of actual usage over 30 days was 61 cores. Memory: 720 GB requested, 410 GB p95 used. Not dramatic in percentage, but the capacity plan said "4 more nodes in Q1 2026" because three namespaces were near their quota and the quota was in requests.

## Method (HF-4705)

1. For every Deployment and StatefulSet, take 30 days of `container_cpu_usage_seconds_total` and `container_memory_working_set_bytes` at 5-minute resolution, compute p95 and p99 per container.

2. New CPU request = p95 × 1.3, rounded up to 50 m. New memory request = p99 × 1.2, rounded up to 64 Mi. Memory limit = request × 1.5 for anything with a garbage collector, equal to request for the rest (the JVM-less, mostly PHP and Rust). CPU limits removed everywhere except the batch jobs (a CPU limit on a latency-sensitive service is throttling you paid for; the HPA note on the kubernetes side had already made the argument).

3. Skip anything with fewer than 30 days of history, anything under 100 m already, and the databases (PostgreSQL, ClickHouse, Redis have their own sizing and their owners).

4. Apply per namespace, one a day, watch `container_cpu_cfs_throttled_periods_total` (should not exist without limits), OOM kills, and the p99 latency of the service for 48 hours.

The script (`finops/rightsize.py`, 300 lines) produces a table and a set of Kustomize patches; a human read every line.

## The table, top of the list

### Largest reductions

| Workload | CPU request before | p95 usage | After | Memory before | After |
|---|---|---|---|---|---|
| `api` (12 replicas) | 2 000 m | 380 m | 500 m | 2 Gi | 1 Gi |
| `notify-worker` (6) | 1 000 m | 90 m | 150 m | 1 Gi | 384 Mi |
| `ingest-svc` (6) | 2 000 m | 1 100 m | 1 500 m | 4 Gi | 4 Gi (kept, batches) |
| `search-indexer` (3) | 1 000 m | 120 m | 200 m | 2 Gi | 768 Mi |
| `tile-server` (4) | 1 000 m | 60 m | 100 m | 1 Gi | 512 Mi |
| `ocr-dispatcher` (2) | 500 m | 30 m | 100 m | 512 Mi | 256 Mi |
| `dispatch-web` (4) | 500 m | 40 m | 100 m | 512 Mi | 256 Mi |
| 33 others | | | | | |

`api` alone freed 18 cores of requests. Its 2 000 m request dated from the first deployment in 2023, when one replica handled everything.

## Results

| | October 2025 | January 2026 |
|---|---|---|
| CPU requested, all namespaces | 158 cores | 98 cores (−38 %) |
| memory requested | 720 GB | 480 GB (−33 %) |
| p95 CPU used | 61 cores | 63 cores (unchanged, as expected) |
| OOM kills in the month | 4 | 6 (two workloads readjusted, see below) |
| nodes' worth of schedulable capacity freed | | about 6 of 22 |
| 2026 node order | 4 nodes | 0 |

Four nodes not bought is 4 × 550 EUR a month of amortisation for four years: 2 200 EUR a month, 105 000 EUR over the cycle, and the [[reserved-capacity-decision-2026-01]] then reduced the pool's margin on top of it. The change shows in the API line of [[per-service-cost-table-q2-2026]] as −1 400 a month because the per-service split uses requests.

## What went wrong

- `notify-worker` at 384 Mi hit OOM during the January OTP burst (a producer bug, not a sizing one, but the memory spike was real). Raised to 512 Mi. The lesson is that p99 over a quiet month is not the p99 of an incident; the multiplier for anything that consumes a queue is 1.5 now.

- `search-indexer` at 200 m CPU took 40 minutes instead of 12 for a full reindex in December. Not a failure, but the search team noticed. It has a `reindex` variant with 1 000 m used for that job and 200 m otherwise.

- One team read "requests lowered" as "we may run more replicas", and doubled a Deployment. Requests total went back up by 3 cores for a week until the monthly review asked why.

## What is now permanent

- `finops/rightsize.py` runs monthly in report mode and posts the ten workloads with the largest gap between request and p95 to the platform channel. Nobody applies it automatically; the report is read at the review ([[finops-monthly-review-feedback]]).

- New Deployments start from a template with 100 m and 128 Mi, and the review asks for measured numbers after a month, not for guesses at creation.

- Requests are the number on the per-service cost table. A team that wants a smaller line lowers its requests, and the report tells it whether it can.
