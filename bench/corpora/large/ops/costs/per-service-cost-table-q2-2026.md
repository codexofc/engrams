---
name: per-service-cost-table-q2-2026
description: Q2 2026 run cost by service from the cost labels: notifications 17 %, warehouse 15 %, API 14 %, storage 11 %, with the allocation method and each owner's lever
type: reference
status: active
verified: 2026-07-08
---

# Cost per service, Q2 2026

## Method

Every euro of [[infra-cost-overview-2026]] is attributed to one service or to `shared`. Three attribution methods, declared per line in `finops/costs.yaml`:

- **direct**: an invoice that belongs to one service (Bipline to notifications, Verifid to onboarding, the ClickHouse support contract to the warehouse).

- **by labels**: cluster compute and storage, split by the `hf.cost/service` label on workloads ([[cost-allocation-labels]]), using the monthly average of CPU requests and memory requests (not usage; you pay for what you reserve, which is the argument of [[rightsizing-2025-q4-requests-limits]]) and of persistent volume size.

- **by share**: colocation, network, licences without a natural owner, split in proportion to the labelled compute of each service.

`shared` is what no service owns: the control plane, the monitoring stack, ArgoCD, the CI. It is shown, not spread, so that it stays visible.

## The table

### Monthly average, April to June 2026

| Service | EUR / month | Share | Main components |
|---|---|---|---|
| notifications | 16 300 | 17 % | Bipline 11 200, Courrix 4 200, compute 900 |
| warehouse | 14 200 | 15 % | 4 data nodes and 3 keeper nodes amortised, cold storage share, backups; the data team's own note has the same figure |
| API (loads, bids, dispatch backend) | 13 400 | 14 % | 9 nodes' worth of requests, PostgreSQL primary and replica, Redis, RabbitMQ |
| storage (object store, backups, offsite) | 10 800 | 11 % | Stashbox amortisation 6 900, offsite 1 900, support 2 000 |
| onboarding and KYC | 7 900 | 8 % | Verifid 6 800, compute 1 100 |
| ML and OCR | 8 600 | 9 % | 2 GPU nodes 2 300, `ml-infer` and feature store compute, `hf-ml-artifacts-prod`, see [[gpu-vs-cpu-inference-cost]] |

### Monthly average, continued: smaller services

| Service | EUR / month | Share | Main components |
|---|---|---|---|
| maps and routing | 6 400 | 7 % | data provider 5 600, tile server compute, CDN share |
| torrent | 3 800 | 4 % | 5 nodes amortised, power |
| dispatch tool and web front | 3 100 | 3 % | compute, CDN share for assets |
| auth | 1 900 | 2 % | 3 pods plus its PostgreSQL share; the cheapest thing on the list for what it protects |

### Monthly average, end: driver backend, shared lines, total

| Service | EUR / month | Share | Main components |
|---|---|---|---|
| driver app backend and sync | 2 700 | 3 % | compute, push relay |
| logging, traces, monitoring (shared) | 2 600 | 3 % | Loki and Tempo storage, Prometheus nodes, the APM licence |
| CI and staging (shared) | 1 800 | 2 % | Skyvale burst VMs, one staging node |
| control plane and cluster overhead (shared) | 2 500 | 3 % | 3 control-plane nodes, ingress, ArgoCD, cert-manager |
| **total** | **96 000** | | |

## What each owner can act on

The table is for owners, not for blame, and each line has a lever that its owner controls and one they do not.

- **Notifications**: the SMS mix. Every SMS moved to push is 3 to 11 cents. The e-mail provider cost is volume-linked and stable. The lever they do not have: the Bipline unit price, which is a contract.

- **Warehouse**: retention of `raw` and the number of shards. Their note says what they will not do (one replica per shard) and why.

- **API**: requests and limits (the rightsizing of Q4 2025 was mostly here), PostgreSQL replica count.

- **Storage**: retention classes, the September drawers. They cannot act on the 31 TB of cold parts, which belong to the warehouse's retention.

- **Onboarding**: Verifid verifications per signup. The fraud ring in May cost 800 EUR of verifications; the pre-KYC checks that now refuse obvious duplicates before calling Verifid are their lever.

- **ML and OCR**: GPU versus CPU, model retrain frequency.

- **Maps**: the data provider contract (renews in 2027) and tile cache hit rate, which is the CDN egress lever.

## Changes since Q4 2025

| Service | Q4 2025 | Q2 2026 | Why |
|---|---|---|---|
| notifications | 18 900 | 16 300 | push-first fallback, −2 100 SMS; dedicated IP, +600 |
| maps and routing | 11 100 | 6 400 | tile cache headers ([[egress-finding-map-tiles-2025-11]]) |
| logging and monitoring | 4 500 | 2 600 | log volume cut ([[logging-cost-reduction-2026]]) |
| CI and staging | 3 900 | 1 800 | staging reduced ([[staging-environment-cost-cut]]) |
| API | 14 800 | 13 400 | rightsizing |
| ML and OCR | 7 400 | 8 600 | second GPU node |
| everything else | within 5 % | | |

## Caveats

- Labels drive the compute split; unlabelled workloads (2 % of requests in June, down from 11 % in October 2025) fall into `shared`. The label coverage is on the finops dashboard and the CI refuses new Deployments without the label since March.

- Requests, not usage: a service with generous requests and low usage looks expensive here, which is the intended pressure.

- The Payla fees are not here. They are 0.9 % of settled volume and belong to finance's margin analysis, not to the run cost.

## Reading your own line

Each service owner gets, with the monthly table, a one-page breakdown of their line: the direct invoices, the labelled compute at 21 EUR per requested core and 2.60 EUR per requested GB, the block storage, the object store buckets they own, and their share of the by-share lines. The page ends with two numbers the owner can move this quarter and one they cannot. For notifications in June 2026: SMS segments per assignment (movable, 0.36 and falling), e-mail sends per bid (movable, 0.29 after the digest), and the Bipline unit price (contract, January). For the warehouse: `raw` retention days and cold-part compression (movable), and the two-replica topology (not movable, decided). The page is generated, the two-plus-one sentence at the end is written by the owner, and the review reads that sentence first.
