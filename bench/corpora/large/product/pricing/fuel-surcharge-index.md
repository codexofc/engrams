---
name: fuel-surcharge-index
description: The fuel surcharge is a monthly percentage derived from the CNR diesel index with a 1.42 EUR/L reference, 0.35 elasticity, published on the 3rd, and frozen per quote
type: reference
status: active
verified: 2026-04-07
---

The `fuel` surcharge (rule 1 of [[surcharge-rules-catalog]]) is a monthly percentage stored in `fuel_index` (`month`, `entity_country`, `diesel_price_eur_l`, `surcharge_pct`, `published_at`).

## Formula

`surcharge_pct = elasticity * (diesel_price - reference_price) / reference_price * 100`

- `reference_price` = 1.42 EUR/L, the average of 2024, set when the formula was introduced (HF-1620). It only moves when the pricing team decides to re-base, which happened once, from 1.51 to 1.42 in January 2025.
- `elasticity` = 0.35: fuel is about 30 to 35 % of a road haulier's cost per kilometre, so a 10 % fuel increase raises cost by about 3.5 %.
- `diesel_price` = the monthly professional diesel price published by the national road transport committee for the entity country (FR, DE, PL, NL each have their own series). Fetched by hand from their sites, the values are entered in the back office by pricing on the 3rd of the month for the previous month.

Result rounded to 0.1 point, clamped to [-5, +25]. April 2026 values: FR +4.3 %, DE +3.8 %, PL +6.1 %, NL +4.0 %.

## Why monthly and not weekly

Carriers plan their pricing per month and shippers budget per month. A weekly index in 2024 caused bids on the same lane to differ by 2 % between Monday and Friday for no reason a shipper could see, and support spent time explaining it. Monthly with a publication on the 3rd is what the industry contracts use (the French indexation clause references the monthly CNR series).

## Frozen per quote

A quote stores the `surcharge_pct` it used. A load posted on the 2nd and bid on the 4th keeps the previous month's rate for bids on quotes issued before publication; new quotes after publication take the new rate. Two bids on the same load can therefore have different fuel bases, which is fine, the bid is the carrier's price.

## Failure modes

- Index not entered by the 5th: `pricing-svc` keeps the previous month and raises `fuel_index.stale` in the pricing channel. Happened in August 2025 (holidays), no impact except the alert.
- A typo (1.72 instead of 1.27 in PL, September 2025) produced +11 % for 40 minutes before rollback. Since then the back office refuses a value more than 15 % away from the previous month without a second confirmation.
