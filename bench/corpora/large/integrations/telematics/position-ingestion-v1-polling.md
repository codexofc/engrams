---
name: position-ingestion-v1-polling
description: The 2024 to 2025 ingestion was a cron per provider in the monolith with row inserts, no dedup, no window filter, 12 min p95 and a 2.9 billion row table
type: reference
status: archived
superseded_by: [[position-ingestion-pipeline]]
verified: 2025-10-10
---

# Ingestion v1 (replaced January 2026)

Kept because the `positions` table it wrote still exists in read-only form until the last partition ages out, and because the reasons it was replaced are the design constraints of the current pipeline ([[position-ingestion-pipeline]]).

## What it was

- A Symfony command per provider (`telematics:poll-trakko`, `telematics:poll-geolyx-legacy`) run by a CronJob every 2 minutes inside the API deployment. Geolyx was polled too; their push webhooks were adopted with the rewrite.

- Each run fetched positions, mapped them by a `LIKE` on the plate (no `tracker_mappings`), and did one `INSERT INTO positions` per row through Doctrine. About 400 rows per second at peak; a 2 minute cron often took more than 2 minutes, and two runs overlapped, which produced duplicates.

- No deduplication: Trakko's sliding window meant every position was inserted about 10 times. A cleanup query ran nightly and deleted rows with identical `(vehicle_id, ts)` keeping the lowest id; it took 40 minutes and locked the table's indexes noticeably.

- No assignment-window filter: every position of every tracked vehicle, all day, every day. 2.9 billion rows by December 2025, going back to 2023, which is the number the compliance review used to make its point.

- Speed was stored. Heading was stored. Raw payloads were stored in a `jsonb` column on the same row.

- Latency: p95 around 12 minutes from the tracker to the dispatcher's map (2 minute cron, plus overlap, plus the ETA cron that ran every 5 minutes on top).

## Why it went

1. Compliance: the window filter and the 90 day retention could not be added sensibly to a table with no partitioning and a nightly dedup that already struggled.

2. Latency: dispatchers measured "the map is 10 minutes behind the driver's phone call" and said so ([[dispatchers-feedback-position-age]]).

3. Coupling: a Trakko API slowdown made the API deployment's cron pods pile up and once exhausted the connection pool shared with the web traffic (October 2025).

## Migration

The gateway ran in parallel for 3 weeks in January 2026 writing to `position_events` while the crons still wrote to `positions`; a daily comparison job checked that every position kept by the new pipeline existed in the old table (it did, plus the old table had the duplicates and the out-of-window rows). Crons stopped 2026-01-20. The old `positions` table was purged to 90 days in the compliance project's first purge run and is dropped when its last partition expires.
