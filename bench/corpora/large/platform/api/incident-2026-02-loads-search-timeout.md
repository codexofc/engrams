---
name: incident-2026-02-loads-search-timeout
description: Feb 2026 incident where GET /v2/loads/search timed out for 48 minutes after a planner flip to a sequential scan on loads, root cause was stale stats after the bulk archive job
type: project
status: active
verified: 2026-02-20
---

# Incident 2026-02-17: loads search timeouts

## Timeline (UTC)

- 06:52 `ApiLatencyP99High` fires for route `api_loads_search`. p99 went from 210 ms to over 10 s.

- 06:58 On-call (backend) confirms 504s from ingress, `php-fpm` slow log full of `LoadSearchRepository::search`.

- 07:05 Dispatch board on the web front shows spinner, dispatchers escalate on the support channel. About 30 shippers affected, all of them using the free-text search.

- 07:14 `EXPLAIN ANALYZE` on the captured query: `Seq Scan on loads` with `rows=4123000`, filter on `status IN ('OPEN','BIDDING')` and `tsv @@ to_tsquery(...)`. The GIN index `idx_loads_tsv` is not used.

- 07:21 `ANALYZE loads;` run by hand on the primary. Took 41 s. Plans flip back to `Bitmap Index Scan on idx_loads_tsv`. p99 back under 300 ms at 07:24.

- 07:40 Incident closed. Total user-visible degradation: 48 minutes.

## Root cause

The nightly archive job (`app:loads:archive`, moves loads terminated for more than 180 days to `loads_archive`) ran its first full-size batch that night after being blocked for two weeks by HF-1490. It deleted 1.9 M rows from `loads` in one run, about 46 % of the table.

Autovacuum's `autovacuum_analyze_scale_factor` was still at the default 0.1, so analyze would have triggered after 10 % of rows changed, which it did, but the analyze started at 06:31 and was still running when the first morning traffic hit at 06:50. Meanwhile the planner used statistics that said `status = 'OPEN'` matched 3 % of rows. After the delete the real fraction was 8 %, and combined with the tsquery selectivity estimate the planner decided a sequential scan was cheaper.

The bigger factor: `n_distinct` for `status` was fine, but the `most_common_vals` histogram had been computed before the delete and no longer matched. The planner's estimate for the AND of the two conditions was off by a factor of 30.

## Fixes

1. HF-1512: `app:loads:archive` now runs `ANALYZE loads` at the end of each run, and batches deletes in chunks of 50 000 with a 2 s pause. Merged 2026-02-18.

2. HF-1513: per-table autovacuum settings on `loads`, `bids`, `load_events`:
   ```sql
   ALTER TABLE loads SET (autovacuum_analyze_scale_factor = 0.02, autovacuum_vacuum_scale_factor = 0.05);
   ```
   Applied via migration `Version20260218101500`.

3. HF-1514: `statement_timeout = '8s'` on the `api` role, so a bad plan produces a 500 with a clear error instead of a 504 after 60 s at the ingress. The mobile app already retries on 5xx with backoff.

4. Dashboard panel "Seq scans per table" added to the PostgreSQL dashboard, from `pg_stat_user_tables.seq_scan` rate.

## What we did not do

We considered `pg_hint_plan` to pin the index. Rejected: it hides the statistics problem and we would forget it is there. We also considered materializing the search into a separate `loads_search` table refreshed by trigger. Deferred, the GIN index is fine when the stats are fine, see [[loads-search-index-gin-trigram]].

## Follow-up

The archive job used to be safe because it never had enough rows to delete. A job whose input size can change by an order of magnitude needs a guard: it now refuses to run if it would delete more than 15 % of the table without `--force`.
