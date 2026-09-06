---
name: shipper-price-cap-rule
description: Shippers can set a max price, bids above it are hidden but the cap value is never shown to carriers, warning under 85 % of suggestion
type: project
status: active
verified: 2026-05-14
---

## What it is

At posting, a shipper can set `loads.max_price_cents`. It is optional, used on 38 % of loads (May 2026), mostly by large shippers who have a contract rate in mind.

## Rules

- Carriers see that a cap exists ("Le chargeur a fixé un prix maximum") but not its value. Showing the value turns every bid into the cap: tested in April 2025 on 2 000 loads, 82 % of bids landed exactly on the cap and the shipper's price went up 6 % on average. Never again.

- A bid above the cap is accepted by `bid-svc` but placed in state `open` with `above_cap = true`; the shipper does not see it in the list, the carrier gets `bid.above_cap` as an info message and can re-bid. About 9 % of bids are above cap; 40 % of those carriers re-bid within 30 minutes.

- If the bidding closes with only above-cap bids, the shipper is shown them with the message "No bid within your maximum, here are the closest ones" and can accept one or re-open. This changed in HF-2360 (November 2025): before that the load just expired and shippers did not understand why they had no bids.

- A cap below 85 % of the suggested price ([[bid-engine-architecture]] quote) shows a warning at posting: "Your maximum is below the usual price for this lane, you may receive few bids." The shipper can keep it. Loads posted with such a cap have a fill rate of 61 % against 93 % overall.

## Interaction with ranking

`BidRanker` ([[bid-ranking-score]]) ignores above-cap bids in its price normalisation, otherwise a cap-crossing bid could not exist without distorting the rest.

## Experiment we did not run

Auto-suggesting a cap at 105 % of the suggestion was proposed. Rejected without an experiment: the anchor experiment ([[experiment-anchor-price-2026-03]]) already showed that shippers and carriers both anchor on what we display, and a displayed cap becomes the price. The cap should stay something the shipper brings from their own knowledge.

## Data

`max_price_cents` is in the loads fact table in the warehouse and used as a feature nowhere; the pricing model must not learn from it (a shipper's cap is their private belief about the price, not the market). This is written in the feature list of [[price-suggestion-model-v2]] as an explicit exclusion.
