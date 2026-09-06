---
name: freshness-monitors
description: Freshness rules compare max(updated_at) of a core table to now, thresholds 15 min for CDC-fed tables, 30 min for batch marts, 26 h for daily, measured every 5 min by dq-runner --schedule, with a quiet-hours rule for tables whose source legitimately stops at night
type: reference
status: active
verified: 2026-05-19
---

# Freshness monitors

## What "fresh" means here

A `freshness` rule ([[dq-framework-overview]]) runs `SELECT max(<column>) FROM <table>` (on ClickHouse this is a metadata read for tables with the column in the sort key or a `minmax` index, 20 ms) and compares `now() - max` to the threshold. It says how old the newest row is, which is the question the dispatch team asks when the dashboard looks stale. The warehouse team's freshness SLO (the target, and how it is reported) is theirs; this note is about the monitor that measures it.

The column is chosen per table: `inserted_at` for tables fed by `ingest-svc` (when the warehouse received it), `updated_at` for CDC-projected tables (when the source row changed), `computed_at` for marts. The difference matters: a CDC table with `updated_at` 30 minutes old on a Sunday morning is normal (nothing changed), the same with `inserted_at` means ingestion stopped. Both columns exist on tier-1 tables and there are two rules on each.

## Thresholds

| Table class | Column | Threshold | Severity | Reason |
|---|---|---|---|---|
| CDC-fed `core` (`loads`, `bids`, `load_status_history`, ...) | `inserted_at` | 15 min | page | `ingest-svc` batches every 30 s, 15 min is 30 missed batches |
| same | `updated_at` | 2 h, business hours only | warn | detects a silent CDC stop when volume is high |
| batch marts (`mart.*`, hourly) | `computed_at` | 90 min | warn | the hourly job plus one retry |
| daily (`feat.*_daily`, `analytics.*`) | `computed_at` | 26 h | warn | the nightly run plus two hours |
| `raw.driver_positions` | `inserted_at` | 5 min | page | 9 000 rows a second, 5 min of silence is a broker or ingest problem |
| `billing.*` projections | `inserted_at` | 4 h, business hours | warn | Payla webhooks are bursty, quiet is normal |

"Business hours only" is `quiet_hours: "22:00-06:00 CET"` on the rule, plus `quiet_days: [sat, sun]` for the billing ones. A rule in quiet hours still runs and records to `dq.results`, it just does not alert. The dashboard shows it grey.

## Schedule

`dq-runner --schedule freshness` runs every 5 minutes from a CronJob on the data namespace, 62 rules in 3 to 4 seconds total. The freshness rules are not tied to a marmot model run (the whole point is to notice when a run did not happen), so they are the one rule kind not in the post-hook path.

## What it has caught

- 2025-12-03: `ingest-svc` stalled on `raw.bids` after the table was restored from a backup with a different partition key and the offset table pointed into the void. `bids_fresh_15m` paged at 03:20; the rebalance storm note on the streaming side has the consumer settings that came out of it, but the detection was this rule.

- 2026-02-19: `mart.dispatch_daily_kpis` not computed because its marmot model failed on a division by zero for a new country with zero loads. `warn` at 90 min; fixed the model.

- 2026-04-02: `core.carriers` stale for 5 hours during business hours (`updated_at` rule): the CDC publication had lost the table after a migration recreated it, and the CDC connector's own check for that did not exist yet. `warn`, but the first signal.

- 2026-05-19: `raw.driver_positions` 6 minutes stale at 06:50 during the broker disk incident. Paged the data on-call, who saw the platform on-call was already on the broker and stood down.

## What it does not catch

A source that keeps writing garbage on time. Freshness is necessary, it is never sufficient, and the SEK invoices ([[missed-2026-04-sek-invoices-summed-as-eur]]) were perfectly fresh for six weeks.

## Tuning

- The 15-minute threshold on CDC tables paged 6 times in October 2025 during the warehouse's 03:00 maintenance window, when inserts pause for 4 to 8 minutes and the batch after catches up. Not a real staleness; raised from 10 to 15 minutes, and the maintenance window is declared as `quiet_hours: "03:00-03:20"` on those rules. Zero false pages since.

- `core.invoices` was `page` at 15 min and paged every Saturday: billing runs in batches at 04:00, 12:00 and 18:00 on weekdays. Moved to `warn` at 30 min on weekdays with quiet weekends. The billing owner agreed that a Saturday morning invoice delay is not worth a page.

- A proposal to compute freshness as "rows expected by now vs rows present" (a volume model) was tried and dropped: it is the anomaly rule ([[volume-anomaly-seasonal-thresholds]]) under another name, and the two kinds should stay distinct so that the alert says which of "nothing arrived" and "fewer arrived than usual" is true.

## Where the number is shown

The `Data quality / Freshness` board has one row per tier-1 table, the current age, the threshold and the last 24 h as a sparkline. It is the board the dispatch team's lead has on a screen, and "is the warehouse up" questions in the channel dropped from a few a week to about one a month after it went up in November 2025.
