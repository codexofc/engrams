---
name: missed-2025-11-duplicate-loads-after-replay
description: Nov 2025, core.loads carried 41 000 duplicate load rows for 9 days after a search-indexer replay was accidentally run on the shared ingest group, row counts looked plausible, dashboards over-counted loads by 1.3 %, caught by a finance reconciliation not by us, the rules and the framework came out of it, HF-4480
type: project
status: active
verified: 2025-12-18
---

# Missed: duplicate loads after a replay, 2025-11-04 to 11-13

## What happened

On 2025-11-04 an offset reset meant for `search-indexer` (rebuilding the search index after a mapping change) was run against the wrong consumer group. The command had `--group ingest-svc` from a copy-pasted history entry. `ingest-svc` re-read three days of `cdc.app.loads` and `cdc.app.bids`. The first hypothesis during the investigation was that the offset-in-the-sink deduplication had failed on the re-read; it had not, a straight replay produces the same batch tokens and ClickHouse drops the duplicate inserts. It took two days to find the real path.

The replay restarted `ingest-svc` with static membership not yet configured (that came a week later, after the rebalance storm), and during the resulting rebalances two members processed overlapping batches whose deduplication tokens were computed from different batch boundaries. The tokens did not match, ClickHouse accepted both inserts, and `raw.cdc_app_loads` gained 41 000 duplicate rows (same `load_id`, same `lsn`, different `inserted_at`). `core.loads` was built with `ReplacingMergeTree` on `(load_id)` but queried without `FINAL` in four of the seven dashboards, and the daily incremental model that fed `mart.dispatch_daily_kpis` counted rows, not distinct loads.

So: a wrong group name, a deduplication token that depended on batch boundaries, and marts that counted rows.

## Why nobody noticed for nine days

- The existing checks ([[legacy-row-count-checks]]) compared today's row count in `core.loads` to yesterday's with a 20 % band. 41 000 on 3.1 M is 1.3 %. Green.

- The freshness checks were fine. Everything was fresh.

- The dashboards moved by 1.3 %, upwards, in a month where loads were growing 4 %. Nobody sees 1.3 %.

- The people who ran the replay looked at `raw._offsets` and at the lag, both fine, and moved on.

## Who noticed

Finance, on 2025-11-13, reconciling "loads delivered in October" from the warehouse against invoices issued: 1 200 more loads than invoices for the last three days of October, when the number is usually within 30. They asked billing, billing asked data, data found the duplicates in an hour once they knew where to look.

## Impact

- Nine days of dispatch KPIs over-counted by 1.3 % on loads and 0.9 % on bids. Monthly report for October re-issued.

- The ETA model's weekly retrain on 2025-11-10 used the duplicated rows; the ML team re-ran it after the fix, the metrics did not move (duplicates of the same rows do not change a tree much), but they had to check.

- Two carrier lateness scores in the November review were computed on the duplicated data. Recomputed, one changed by 0.2 points, nobody's tier moved.

- 41 000 rows deleted from `raw.cdc_app_loads` (with `ALTER TABLE ... DELETE WHERE` on the duplicate `inserted_at` values, 20 minutes), `core.loads` rebuilt (`marmot run --full-refresh core.loads`, 40 minutes), marts rebuilt for 9 days.

## What came out of it (HF-4480)

1. **A framework, not more scripts.** The check that would have caught this in the first hour is `unique (load_id)` on `core.loads` with `FINAL` semantics, plus `count(*) = count(distinct load_id)` on `raw` grouped by day. Neither existed because each check was a script someone wrote once. `dq-runner` ([[dq-framework-overview]]) was written in the following three weeks with `unique` as its first rule kind.

2. **Reconciliation as a rule kind.** Finance's "loads delivered vs invoices issued" became `loads_delivered_vs_invoices` (`reconciliation`, `warn` at 50, `page` at 200), and the pattern was extended to CDC counts ([[dq-rule-catalog-core]] has `loads_count_vs_cdc`) and to Payla ([[reconciliation-payla-settlements]]).

3. **Deduplication tokens from content, not boundaries.** `ingest-svc` computes the token from `(topic, partition, first offset, last offset)` since 2025-11-20, so two members processing the same offsets produce the same token whatever the batch shape. Rebalances cannot duplicate any more.

4. **Marts count distinct.** All 20 `mart.*` models were reviewed for `count(*)` on tables with a replacing engine; six were changed to `count(distinct)` or to reading with `FINAL`.

5. **The replay checklist** on the streaming side gained its point 7 ("note the current offsets") and the group name must be typed in the ticket before the command; the command is generated from the ticket, not pasted.

## The row count table that fooled everyone

| Day | `core.loads` rows | Day-over-day | Distinct `load_id` | Difference |
|---|---|---|---|---|
| 11-03 | 3 102 400 | +0.3 % | 3 102 400 | 0 |
| 11-04 | 3 144 900 | +1.4 % | 3 103 800 | 41 100 |
| 11-05 | 3 146 300 | +0.04 % | 3 105 200 | 41 100 |
| 11-13 | 3 158 100 | +0.1 % | 3 117 000 | 41 100 |

The fourth column is the check that did not exist. It is a 200 ms query. This table is on the first page of the framework's documentation, because it says everything about why counting rows is not a quality check.

## Lesson written in the review

"Plausible" is the most dangerous state of a number. A metric that moves by 40 % gets looked at; one that moves by 1.3 % in the right direction gets believed. Quality rules must test invariants (a load appears once, an invoice has one currency, an assigned load has a carrier), not plausibility.
