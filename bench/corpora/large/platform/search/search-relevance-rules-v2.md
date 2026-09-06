---
name: search-relevance-rules-v2
description: Carrier search ranking since HF-3200: filters then a function score on distance, date, price vs lane median, freshness, fit; the 300 km rule; tests
type: reference
status: active
verified: 2026-08-10
---

# Relevance rules, version 2

Replaces [[search-relevance-rules-v1]] since 2026-07-08 (HF-3200). The query has two stages: a **filter** stage that decides what is eligible, and a **score** stage that orders it. Filters never affect the score; scores never exclude.

## Filters (must match)

- `searchable = true`, `bidding_closes_at > now`

- `visibility = PUBLIC`, or `PRIVATE` with the carrier in the shipper's favourites (a terms filter on `shipper_org_id` with the carrier's favourited shippers list, at most 200)

- `shipper_org_id != carrier's own org` (logistics companies that are both)

- pickup within the radius of the search point ([[search-geo-radius-queries]]); delivery within the delivery zone if one is given

- pickup window overlapping the requested date range (default today to +7 days)

- `vehicle_types` intersecting the carrier's declared fleet, unless the carrier unticks "my fleet only"

- ADR classes: excluded unless the carrier's fleet declares ADR

- weight, pallets and ldm under the carrier's largest declared vehicle

## Score

A `function_score` over the filtered set, `score_mode: sum`, `boost_mode: replace` (the text query's own score is ignored; text is a filter here).

### Components

| Component | Function | Weight | Notes |
|---|---|---|---|
| distance | gauss decay, origin = search point, scale = radius / 2, offset = 10 km | 3.0 | drops smoothly, half at radius / 2 |
| pickup date | gauss decay on `pickup.window_start`, origin = requested start, scale = 2 days | 2.0 | tomorrow beats next week |
| price per km | linear on `price_per_km_eur_cents / lane_median_cents`, clipped to [0.6, 1.4], mapped to [0, 1] | 2.0 | above-median lanes score higher; "on request" loads get 0.5 |
| freshness | exp decay on `published_at`, scale = 12 h | 1.0 | new loads get a small push |
| fit | `saved_search_fit` field, 0 to 1 | 1.5 | how much the load matches the carrier's saved searches, computed at query time from the carrier's profile |
| bid pressure | `1 / (1 + bid_count)` | 0.5 | loads with fewer bids surface slightly |

Total maximum 10. The weights are in `ranking/v2.yaml`, deployed with the search API, not hard-coded.

The lane median (`lane_median_cents`) comes from the pricing service's daily export per country-pair and distance band, loaded into the search API's memory at startup and refreshed hourly. A missing lane falls back to the country-pair median, then to the European median.

## The 300 km rule

When the radius is above 300 km (a carrier searching "anywhere in Germany"), the distance weight drops to 1.0 and the price weight rises to 3.0. The Ravello feedback in the support cases showed that a wide radius with distance-dominant ranking buries a well-paid load 250 km away under fifty poorly paid loads 60 km away; a carrier who widened the radius has told us distance matters less to them. Tested in the [[search-ab-testing-ranking]] June experiment: bids per search up 9 % for wide searches, no change for narrow ones.

## What v2 changed from v1

- Price against the lane median instead of raw price per km (v1 favoured long expensive loads regardless of lane).

- The fit component (new).

- The 300 km rule (new).

- Freshness scale from 6 h to 12 h and weight halved: v1 made the list reshuffle too much between two refreshes.

- Bid pressure (new, small).

## Testing

`tests/ranking/cases/*.yaml`: each case is a carrier profile, a search, a fixed set of documents, and the expected order of the top 5. 63 cases. They run against a local single-node haystack in CI. A weight change that reorders a case is a deliberate decision with the case updated in the same PR. There is also an offline replay: yesterday's 20 000 searches re-scored with the candidate weights, reporting how often the load the carrier actually clicked moved up or down (`ranking-replay` command). It is a proxy, not a truth, and it caught the v1 freshness problem.

## What is not in the score

Carrier rating (it is the shipper's signal, not the carrier's), shipper rating (we do not have one), sponsored placement (no paid ranking; decided and written in the product notes), and text relevance (a carrier typing "palettes" filters, it does not rank).

## Worked example

A carrier in Lyon, fleet `TAUTLINER` and `FRIGO`, saved searches "Lyon to Milan" and "Lyon to Barcelona", searches with radius 150 km, dates next 7 days. Two candidate loads:

- Load A: pickup 40 km away, tomorrow, Lyon to Milan, 1.45 EUR/km on a lane whose median is 1.30, published 3 hours ago, 2 bids. Components: distance 2.9, date 1.9, price about 1.6 (ratio 1.12 mapped inside the clipped range), freshness 0.8, fit 1.5, bid pressure 0.17. Total about 8.9.

- Load B: pickup 120 km away, in 5 days, Lyon to Hamburg, 1.70 EUR/km on a lane whose median is 1.65, published 10 minutes ago, 0 bids. Components: distance 1.1, date 0.4, price about 1.1, freshness 1.0, fit 0, bid pressure 0.5. Total about 4.1.

A ranks first, and would have in v1 too. Change B to 0.95 EUR/km on a 0.80 lane and it climbs to about 5.5 but still trails; in v1, B's raw 1.70 EUR/km would have given it the full price component and it would have sat within a point of A despite being off the carrier's lanes. This is the case `tests/ranking/cases/lyon-two-loads.yaml`.

## Tuning notes

The weights are not the result of an optimiser; they were set by hand from the replay and then checked online. A learned ranking was discussed and shelved: 20 000 searches a day is thin for it, the features are few, and a hand-set weight is something the support team can explain to a carrier. Revisit when the search log has a year of data.
