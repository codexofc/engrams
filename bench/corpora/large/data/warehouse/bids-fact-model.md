---
name: bids-fact-model
description: core.bids: one row per bid with the quote and surcharges as columns, sharded with loads by load_id, and the is_initial flag
type: reference
status: active
verified: 2026-05-30
---

`core.bids` is one row per bid, final state, built every 10 minutes by the marmot model `bids` ([[marmot-model-runner]]) from `raw.cdc_app_bids` and `raw.cdc_app_quotes`. Sharded by `cityHash64(load_id)`, same as `core.loads` ([[loads-fact-model]]), so a join between the two never leaves the shard.

## Columns

- Identity: `bid_id`, `load_id`, `carrier_id`, `quote_id`.

- Amounts: `amount_cents`, `currency`, `amount_eur_cents` (converted at the daily rate of `created_at`, from the `fx_rates` dictionary).

- State: `state` (`open`, `withdrawn`, `accepted`, `declined`, `expired`, `outbid`), `decline_reason`, `created_at`, `final_state_at`, `above_cap`, `flagged_junk`.

### Quote and surcharge columns

- Quote context, copied from the quote the carrier saw: `suggested_cents`, `base_cents`, `lane_confidence`, `config_version`.

- Surcharges flattened: `surcharge_fuel_cents`, `surcharge_weekend_cents`, `surcharge_night_cents`, `surcharge_adr_cents`, `surcharge_temperature_cents`, `surcharge_urgent_cents`, `surcharge_toll_cents`, `surcharge_ferry_cents`, `surcharge_multi_stop_cents`, plus `surcharge_total_cents` and `breakdown_consistent` (true when the carrier's breakdown sums to the bid amount within 1 cent).

### Derived flags

- `is_initial`: true for the carrier's first open bid on the load, false for re-bids after being outbid or withdrawing. Computed by window over (`load_id`, `carrier_id`) ordered by `created_at`.

- `rank_at_close`: the bid's rank in the shipper's list when bidding closed, taken from the `bid.ranked` product event; null before February 2026.

## Why the surcharges are columns and not a map

A `Map(String, Int64)` was the first design. Analysts wrote `surcharges['fuel']` and it worked, but a typo (`surcharges['fuell']`) silently returns 0 in ClickHouse and one analysis in November 2025 reported a fuel surcharge share of 0 % for a month. Nine explicit columns are ugly and safe; a new surcharge rule means a schema migration ([[schema-migration-process]]), which is fine, the rules change once a year.

## `is_initial` and experiments

Pricing analyses use initial bids only. The flag is computed in the warehouse, not in the app, because the app's notion (state `withdrawn` then a new bid) does not distinguish a re-bid from a carrier changing their mind before anyone else bid. Here, initial means first by time per carrier and load, full stop. The pricing team's experiment guidelines refer to this column.

## Volumes

730 000 bids a month, 0.62 bids per row of `raw.cdc_app_bids` (each bid has about 1.6 CDC rows, creation plus state change). The table is 41 GB compressed for the 400-day window.

## Known issues

- `amount_eur_cents` for CZK bids before March 2026 used a placeholder rate of 25.0; corrected by the backfill of 2026-03-18 ([[backfill-runbook]]).
- `rank_at_close` is missing for about 2 % of bids where the `bid.ranked` event arrived after the model ran and the load partition was already compacted; see [[late-arriving-events]].
