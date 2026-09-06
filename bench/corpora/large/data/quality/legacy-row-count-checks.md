---
name: legacy-row-count-checks
description: Until Oct 2025 data quality was 14 shell scripts comparing today's row count of a core table to yesterday's within 20 %, plus a freshness ping, run from a cron on ops-tools, green through both misses of 2025, replaced by dq-runner and its anomaly rules
type: reference
status: archived
superseded_by: [[volume-anomaly-seasonal-thresholds]]
verified: 2025-10-20
---

# The row count checks (archived)

What passed for data quality before the framework. Kept because "why don't we just compare row counts" comes up every time someone new joins, and this is the answer. The replacement for this specific idea is [[volume-anomaly-seasonal-thresholds]]; the framework around it is described elsewhere in this project.

## What existed

Fourteen shell scripts in `ops-tools:/opt/dq/`, one per `core` table that someone had once worried about, run at 07:00 by cron. Each did:

```
today=$(clickhouse-client -q "SELECT count() FROM core.loads")
yesterday=$(cat /var/lib/dq/core.loads.last)
# alert if abs(today - yesterday) / yesterday > 0.20
echo $today > /var/lib/dq/core.loads.last
```

Plus one script checking that `max(inserted_at)` on `raw.cdc_app_loads` was less than an hour old, and one checking that the nightly marmot run had written its completion marker.

Alerts went to a channel nobody had muted but everybody had stopped reading, because the 20 % band fired every Monday (weekend versus weekday volume) and never otherwise.

## What it caught

In two years: one occurrence, in 2024, when a marmot model failed silently and `core.bids` had zero new rows for a day (100 % below yesterday). That is a freshness failure wearing a row count costume, and the freshness rule would have caught it two hours earlier.

## What it did not catch

- 41 000 duplicate loads for nine days in November 2025: 1.3 % growth, inside the band ([[missed-2025-11-duplicate-loads-after-replay]]).

- A `core.carriers` dimension with two current rows for 400 carriers for three weeks in 2025: row count up 0.5 %.

- A month of `core.invoices` where `currency` was null for a new shipper's invoices: same row count, wrong content.

- Every Monday, it caught the weekend.

## Why counting rows is not a quality check

A row count is one number about a table. It cannot say whether the rows are the right rows, whether they are unique, whether the columns are filled, or whether the count is where it should be for this weekday in this country. It moves when the business moves, and the business moves more than most bugs do. The scripts measured plausibility, and plausibility is the state in which a wrong number is believed.

## What replaced it, in order

1. `unique` rules on every primary key (the first rule kind written).

2. `freshness` rules with per-table thresholds and quiet hours.

3. `not_null` and `accepted_values` for content.

4. Anomaly rules with a seasonal band per segment for the volume question the scripts were trying to ask.

The scripts were deleted on 2025-10-28, the day the last of the 14 tables had a `unique` and a `freshness` rule. `/var/lib/dq/` on `ops-tools` is gone. The channel was archived.
