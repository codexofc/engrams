---
name: carrier-dimension-scd
description: core.carriers as a versioned dimension with valid_from and valid_to, five versioned attributes, ASOF JOIN pattern, daily score apart
type: project
status: active
verified: 2026-04-29
---

## Why versioned

A carrier's fleet size, verification status, reliability score and country of registration change over time, and analyses need the value at the time of the load, not today's. "How do carriers with fewer than 3 trucks bid" must use the fleet size on the day of the bid; a carrier with 40 trucks today had 2 in 2024.

## Model

`core.carriers` has one row per carrier version: `carrier_id`, `version`, `valid_from`, `valid_to` (null for the current version), and the attributes. A new version is created when any of the five versioned attributes changes:

1. `fleet_size_band` (`1_3`, `4_20`, `21_plus`), from `declared_trucks`
2. `verification_level` (0 to 5, the highest step passed in the onboarding verifications)
3. `licence_scope` (`community`, `national`, `light_vehicle`, `none`)
4. `country`
5. `self_billing` (mandate signed or not)

Non-versioned attributes (name, created_at, referral source, `is_internal`) are carried on every version with the current value.

Built by the marmot model `carriers` ([[marmot-model-runner]]) as a `full` materialisation every hour from `raw.cdc_app_carrier_accounts` and `raw.cdc_app_carrier_verifications`: the CDC history gives every change with its `ts_ms`, the model collapses consecutive changes that do not touch a versioned attribute, and assigns `valid_from` and `valid_to`. 71 000 carriers, 310 000 versions, 2 minutes to rebuild.

## Joining

`core.loads` and `core.bids` do not carry the carrier attributes. The join is:

```
FROM core.bids b
ASOF LEFT JOIN core.carriers c
  ON b.carrier_id = c.carrier_id AND b.created_at >= c.valid_from
```

`ASOF JOIN` picks the latest version whose `valid_from` is before the bid. The `marts.bids_enriched` table does this join once an hour so analysts do not have to; the [[query-guidelines-analysts]] note says to use the mart unless the analysis needs a non-standard timestamp.

The `ASOF` join needs `core.carriers` sorted by (`carrier_id`, `valid_from`), which is its `ORDER BY`; joining the other way round (carriers as the left side) is 30 times slower.

## The reliability score is not here

The carrier reliability score used by the bid ranking changes daily and would create 71 000 versions a day. It is a separate table `core.carrier_reliability_daily` (`carrier_id`, `date`, `score`, components), 26 M rows, joined by date. This split (slow attributes versioned, fast attributes daily) is the rule for any future dimension.

## Backfill note

Because the model is `full`, a backfill is just a run. But the CDC history in `raw` only goes back 400 days ([[retention-rules]]); versions older than that were frozen into `core.carriers_history_frozen` in October 2025 and the model unions them. If a carrier's history before October 2025 looks odd, that is where to look.
