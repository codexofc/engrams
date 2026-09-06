---
name: carrier-rating-computation
description: Carrier score (0 to 100) is recomputed nightly by app:carriers:rescore from the last 90 days, weights 40 on-time, 30 POD delay, 20 cancellations, 10 shipper feedback, stored in carrier_scores
type: project
status: active
verified: 2026-04-28
---

# Carrier rating

The score shown to shippers next to each bid. Product owns the formula, we own the pipeline. Current formula is version 3, effective 2026-03-01 (HF-1560).

## Inputs (rolling 90 days, per carrier)

| Component | Weight | Source | Definition |
|---|---|---|---|
| On-time delivery | 40 | `load_events` kind `DELIVERED` vs `loads.delivery_window_end` | share of loads delivered before window end, with a 30 min tolerance |
| POD delay | 30 | `load_events` kind `POD_UPLOADED` vs `DELIVERED` | share of PODs uploaded within 2 h of delivery |
| Cancellations | 20 | `bids` with `status = 'WITHDRAWN'` after acceptance | 100 minus 25 per cancellation, floor 0 |
| Shipper feedback | 10 | `shipper_feedback.rating` (1 to 5) | average scaled to 0 to 100 |

Score = weighted sum. A carrier with fewer than 5 completed loads in the window has `score IS NULL` and the UI shows "Nouveau" instead of a number. This threshold was 3 in version 2 and shippers complained that one bad delivery made a new carrier look terrible.

## Pipeline

`bin/console app:carriers:rescore` runs from a CronJob at 02:10 Europe/Paris. It takes about 4 minutes for 3 800 carriers. It writes to `carrier_scores (carrier_id, computed_at, score, components jsonb, formula_version)` and never updates in place: one row per run, so we can show history and debug "why did my score drop". A retention job keeps 400 days.

`carriers.current_score` is a denormalized copy updated in the same transaction, indexed, because the bid list sorts on it.

The query is a single CTE per component, `EXPLAIN` shows no seq scan on `load_events` thanks to `idx_load_events_carrier_kind_at (carrier_id, kind, occurred_at)`. Watch that index, it is the one that makes this job 4 minutes instead of 40.

## Why nightly and not event-driven

We tried real-time updates in version 1 (a Messenger handler on each `DELIVERED` event). Two problems: the score bounced during the day and carriers called support about it, and the handler recomputed the full 90-day window each time, 3 800 carriers times a dozen events a day. Nightly is boring and boring is fine here.

## Known distortions

- A carrier who only takes short urban loads has an easier on-time component. Product accepted this.
- Timezones: `delivery_window_end` is stored in UTC, the tolerance is computed in UTC. Fine, since both sides are instants. See [[timezone-handling-utc-rule]].
- Loads cancelled by the shipper are excluded from every component. A cancelled load does not count against anyone.

## Next

Version 4 (HF-1610, not scheduled) wants to add GPS-based ETA accuracy from the driver app. Needs the tracking data from the data platform, not in the API database.
