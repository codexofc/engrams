---
name: experiment-min-bid-floor
description: HF-2290: a 70 % minimum bid floor was rejected, it cut bids 19 % on low-confidence lanes and was wrong more often than carriers
type: project
status: active
verified: 2025-12-18
---

## Hypothesis

Shippers complained about absurdly low bids (a Warsaw to Lyon full truck at 380 EUR) that they had to wade through. The proposal was to refuse any bid under 70 % of the suggested price with the message `bid.below_floor`.

## Setup

Flag `pricing.min_bid_floor`, 50/50 by `load_id` this time (the treatment is on the load, not the carrier), 2025-10-20 to 2025-11-30, all entities. Floor computed from the same quote as the suggestion (see [[bid-engine-architecture]]). Primary metric: shipper-reported "junk bid" flags per load (the thumbs-down on a bid, event `bid.flagged_junk`). Secondary: bids per load, fill rate, awarded price.

## Results

| Metric | Control | Floor |
|---|---|---|
| junk flags per 100 loads | 4.1 | 1.9 |
| bids per load (median) | 6.0 | 5.2 |
| fill rate | 93.1 % | 91.0 % |
| awarded price / suggestion | 1.041 | 1.049 |

Junk flags halved, as expected. But bids per load fell 13 % overall and 19 % on low-confidence lane clusters ([[pricing-lane-clusters]]), fill rate lost two points, and the awarded price went up. On low-confidence lanes the suggestion is often too high, so the floor was rejecting carriers who were right.

We sampled 200 rejected bids by hand. 63 were plausible prices for the lane (backhaul, carrier already positioned nearby), 91 were plausible but aggressive, 46 were junk. The floor is a worse judge than the shipper.

## Decision

Rejected. Instead:

- Bids under 70 % of the suggestion are accepted but shown in a collapsed "low bids" group on the shipper side, with a one-line explanation. Shipped as HF-2312 in December 2025, junk flags at 2.3 per 100 loads in January 2026 without loss of bids.
- Carriers whose bids get flagged junk more than 5 times in 30 days get a warning, and after 10 their bids are hidden by default for a month. Nine carriers were affected in the first quarter.

## Lessons folded into the guidelines

- Randomising by load and not by carrier was right here and would have been wrong for the anchor experiment; the unit of randomisation must be the unit that receives the treatment, see [[pricing-experiment-guidelines]].
- A metric that only counts complaints (junk flags) is easy to improve by removing what people can complain about. Always pair it with a volume metric.
