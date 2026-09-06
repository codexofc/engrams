---
name: logging-cost-reduction-2026
description: Jan to Mar 2026: log volume cut from 1.8 TB a day to 620 GB by dropping four noisy sources and sampling access logs, retention kept, −2 550 EUR/month, HF-4720
type: project
status: active
verified: 2026-04-14
---

# Cutting the log volume

## Where we started

In December 2025 Loki ingested 1.8 TB a day, 54 TB a month at 30 days retention, on 3 dedicated nodes (ingesters and queriers) plus the object store for chunks. The logging line in [[infra-cost-overview-2026]] was 2 600 EUR a month in Q4 2025, plus about 1 900 of node amortisation that the cost table attributed to `monitoring`. Queries over more than 6 hours timed out routinely, which was the complaint that started the work, not the cost.

The observability project's own note on log volume had already said what the big sources were. This note is about what was done and what it saved.

## The sources, December 2025

### Top log producers by volume

| Source | GB / day | Share | Nature |
|---|---|---|---|
| ingress-nginx access logs | 620 | 34 % | one line per request, 18 000 req/s at peak, JSON with 30 fields |
| `api` PHP application logs | 380 | 21 % | `INFO` on every request, plus a `DEBUG` channel left on in prod for one bundle |
| `driver-backend` | 210 | 12 % | position uploads logged at `INFO` with the payload |
| CoreDNS | 140 | 8 % | `log` plugin on all queries, from a 2024 debugging session |
| torrent brokers | 110 | 6 % | request logs at `DEBUG` on two of five brokers (an upgrade left them) |
| `ingest-svc` | 90 | 5 % | batch summaries, fine, plus a per-row type violation log |
| everything else | 250 | 14 % | |

## What was done (HF-4720)

1. **Ingress access logs**: kept, but sampled and aggregated. The full line is written for every 4xx and 5xx and for 1 in 50 of the 2xx (deterministic on the request id so that a trace can be followed). Aggregates per route, status and minute go to Prometheus, which is where anyone actually looked at request rates. 620 to 45 GB a day. The one thing lost: reconstructing a specific user's exact sequence of successful requests from access logs. That is what traces are for, and Tempo has 7 days.

2. **`api`**: the `DEBUG` channel was a Monolog handler for the payments bundle, enabled in 2024 for a Payla webhook issue and never disabled. Off. Request-level `INFO` lines reduced to one per request with the essentials (route, status, duration, user id hash, trace id); the six lines per request that said "entering handler", "leaving handler" removed. 380 to 110 GB.

3. **`driver-backend`**: position payloads out of the logs. The position is on the topic and in the warehouse; logging it a third time at `INFO` served nobody. Kept a one-line summary per batch. 210 to 20 GB.

4. **CoreDNS**: `log` only for `SERVFAIL` and `NXDOMAIN` on internal zones, which the DNS tuning note on the kubernetes side describes. 140 to 3 GB.

5. **torrent brokers**: request logging back to `INFO` on the two brokers. 110 to 25 GB.

6. **`ingest-svc`**: type violations counted in a metric and written to `raw._skipped` (which already existed); the per-row log line removed. 90 to 30 GB.

7. **Loki**: retention kept at 30 days (the argument for shorter was cost, and the cost went away with the volume; the argument for 30 is that incident reviews happen within a week and audits within a month). Ingesters from 3 nodes to 2, and the third node returned to the pool.

## Results

| | December 2025 | March 2026 |
|---|---|---|
| ingested | 1.8 TB / day | 620 GB / day |
| stored (30 days, compressed) | 16 TB | 5.4 TB |
| Loki nodes | 3 | 2 |
| p95 query over 24 h | 28 s (often timeout at 6 h) | 4 s |
| logging cost line | 2 600 EUR / month | 700 |
| node amortisation attributed to monitoring | 1 900 | 1 250 |
| total saving | | about 2 550 EUR / month |

The per-service table ([[per-service-cost-table-q2-2026]]) shows the monitoring line going from 4 500 to 2 600; the difference between that and the 2 550 here is the APM licence, unchanged.

## What nobody missed

We asked, in the channel and at the monthly review ([[finops-monthly-review-feedback]]), for two months, whether anyone had looked for a log line that was gone. Two reports: one person wanted the full access log for one IP during the credential stuffing investigation in the auth project's review (the 4xx lines were all there, which was the relevant part), and one wanted `driver-backend` payloads to debug a position parsing issue (the payload was on the topic, `torrent-console-consumer` gave it in a minute). Neither asked for the logs back.

## What is now permanent

- A per-source daily volume panel with a `warn` at +50 % day over day for any source above 20 GB a day. It has fired twice, both times a deploy that turned `DEBUG` on somewhere; both fixed the same day. This is the alert that would have caught the payments bundle in 2024.

- `DEBUG` in production is a deploy-time flag with an expiry (`LOG_DEBUG_UNTIL=<date>`, the logger ignores it after), not a config line.

- Any new service's logging is reviewed for "what would you search for in this line" before it ships; a line nobody would search for is a metric or nothing.
