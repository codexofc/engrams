---
name: positions-table-partitioning
description: position_events is range-partitioned by day, 92 partitions kept, created 7 days ahead, unique (vehicle_id, ts, provider) per partition, 9 M rows a day
type: project
status: active
verified: 2026-04-06
---

# `position_events` partitioning (HF-2059)

The table behind the pipeline ([[position-ingestion-pipeline]]). Designed for two operations that the old `positions` table ([[position-ingestion-v1-polling]]) made painful: dropping old data, and looking up the recent positions of one vehicle.

## Layout

```sql
CREATE TABLE position_events (
  id            bigint GENERATED ALWAYS AS IDENTITY,
  vehicle_id    uuid        NOT NULL,
  assignment_id uuid        NOT NULL,
  provider      text        NOT NULL,
  ts            timestamptz NOT NULL,      -- trusted time (server or device after gap rule)
  ts_device     timestamptz,
  time_source   text        NOT NULL,      -- 'device' | 'server'
  lat           double precision NOT NULL,
  lon           double precision NOT NULL,
  heading       smallint,
  accuracy_m    smallint,
  ignition      boolean,
  odometer_m    bigint,
  ingested_at   timestamptz NOT NULL DEFAULT now()
) PARTITION BY RANGE (ts);
```

No speed column, by the compliance decision. No raw payload column; the raw object lives 7 days in `hf-telematics-raw` and `id` is enough to find it in the debugging window.

Partitions are **daily**: `position_events_2026_04_06` for `[2026-04-06, 2026-04-07)`. Daily rather than monthly because the retention is 90 days and a monthly partition would keep up to 120 days; daily lets the purge be exact to the day. About 9 million rows and 1.1 GB (with indexes) per day; 92 partitions online (90 days plus 2 of slack).

## Indexes, per partition

- `UNIQUE (vehicle_id, ts, provider)`: the idempotency guarantee behind the dedup cache ([[position-dedup-rules]]), added after [[incident-2025-11-geolyx-duplicate-flood]]. Also the index that serves "last positions of vehicle X" queries, which always carry a `ts` range.

- `BRIN (ts)` with `pages_per_range = 32`: cheap, and enough for the range scans of the ETA backfill and the warehouse export, which read whole time windows.

- `(assignment_id, ts)`: for the tracking page's "positions of this load" query. Considered dropping it in favour of the vehicle index plus a filter; measured 4 times slower for long assignments, kept.

Nothing on `lat, lon`. We never query positions by area; the geofence consumer works from the stream, not the table.

## Creating and dropping

- `telematics:partitions:ensure` runs daily at 01:00 and creates partitions up to **7 days ahead**. If it fails for a week nobody notices until inserts start failing, so it also emits `telematics_partitions_ahead` and an alert fires under 3.

- The default partition exists but is alerted on: any row landing there means a timestamp outside the created range (the plausibility stage should have rejected it). Two rows in six months, both from a harness run against the wrong database.

- Dropping is the compliance project's purge job, not ours: `RetentionRule positions_raw_90d` detaches and drops the partition older than 90 days at 04:00, after writing the day's `trip_summaries` (a job in the data platform that runs at 03:00 on the partition about to expire). The ordering dependency is a check in the purge rule: it refuses to drop a partition whose summary job has not written a completion marker.

## Writing

`COPY` per batch of 2 000 into the parent; Postgres routes to the partition. Measured 18 000 rows per second on the production instance with the three indexes, well over the 110 per second average and the 2 400 per second peak. On a unique violation the batch falls back to `INSERT ... ON CONFLICT (vehicle_id, ts, provider) DO NOTHING` row by row; this path runs a few times a day and is counted.

## Reading patterns and their cost

| Query | Partitions touched | p95 |
|---|---|---|
| last position of one vehicle | 1 (today) | 2 ms |
| positions of one assignment (typ. 8 h) | 1 or 2 | 15 ms |
| all positions of one day for the warehouse | 1, sequential | 40 s |
| one vehicle over 30 days (support) | 30 | 300 ms |

## What we did not do

- Sub-partition by provider or by carrier. Queries never filter on those first.

- TimescaleDB or a time-series store. Plain partitioning does everything we need, and the ops team runs one kind of database.

- Keep an aggregated table ourselves. `trip_summaries` belongs to the data platform, which has the model for stops and rests; we only make sure the raw partition is not dropped before they are done.
