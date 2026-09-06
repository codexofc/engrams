---
name: bid-ranking-score
description: Bids ranked by 0.55 price, 0.25 reliability, 0.15 ETA fit, 0.05 recency in BidRanker, price normalised on the lowest open bid
type: reference
status: active
verified: 2026-02-24
---

The shipper sees bids ordered by `BidRanker.score()`, not by price alone. Sorting by price only was the original behaviour and led shippers to accept the cheapest bid, then complain about no-shows (the no-show rate on cheapest-bid awards was 5.8 % against 2.9 % overall in mid-2025).

## Formula

`score = 0.55 * price_component + 0.25 * reliability + 0.15 * eta_fit + 0.05 * recency`

- `price_component = lowest_open_bid / bid_amount`, so the lowest bid scores 1.0 and a bid 20 % higher scores 0.83. Recomputed whenever a new bid arrives, which is why ranks move.
- `reliability`: the carrier's score from the carrier profile, in [0, 1]: on-time pickup rate (weight 0.5), no-show rate inverted (0.3), document completeness (0.2), over the last 180 days, with a prior of 0.7 for carriers with fewer than 5 loads.
- `eta_fit`: 1.0 if the carrier's declared availability lets them reach the pickup within the window, decreasing linearly to 0 at 4 hours late, using the ETA model of the data team.
- `recency`: 1.0 for a bid less than 10 minutes old, down to 0 at 24 hours. Small on purpose, only breaks ties.

The weights are constants in `BidRanker` with a test that fails if they do not sum to 1. They are not flags: changing the ranking is a product decision that goes through a written experiment ([[pricing-experiment-guidelines]]), not a config change at 18:00.

## What the shipper sees

The list shows the score as a 1 to 5 star display (`round(score * 5)`) and the reason for the top bid ("best price", "most reliable", "arrives earliest"). Bids under 70 % of the suggestion are collapsed in a "low bids" group, see [[experiment-min-bid-floor]].

The shipper can switch to "sort by price" and 27 % do. The awarded bid is the top-ranked one 58 % of the time, the cheapest 31 %, something else 11 %.

## Known weaknesses

- `reliability` is per carrier, not per driver; a 40-truck fleet averages good and bad drivers.
- A new carrier with the 0.7 prior ranks above an established carrier at 0.65. Onboarding asked to keep it that way to give new carriers a chance; pricing wanted 0.6. Compromise at 0.7 with a review after 2 000 awarded loads by new carriers, due around July 2026.
- `price_component` normalised against the lowest bid means one junk bid at 380 EUR crushes everybody's price component. Since HF-2312 the normalisation ignores bids in the collapsed low group.
