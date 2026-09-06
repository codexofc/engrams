---
name: bid-engine-architecture
description: The bid engine is the bid-svc service, bids are immutable rows in bids with a state column, a load closes by first-accept or by expiry, and the suggested price comes from pricing-svc over gRPC with a 150 ms budget
type: reference
status: active
verified: 2026-05-05
---

## Services

- `bid-svc` (Kotlin) owns loads' bidding state and the `bids` table. Endpoints under `/api/v1/loads/{id}/bids`.
- `pricing-svc` (Python) computes the suggested price, surcharges and the shipper cap. Called by `bid-svc` over gRPC (`PricingService.Quote`), budget 150 ms, fallback to the last cached quote for the lane if the call times out (cache key `lane_cluster_id + vehicle_type`, TTL 10 minutes).
- The ranking of bids shown to the shipper is computed in `bid-svc` with the score of [[bid-ranking-score]].

## Data model

`bids`: `id`, `load_id`, `carrier_id`, `amount_cents`, `currency`, `state` (`open`, `withdrawn`, `accepted`, `declined`, `expired`, `outbid`), `created_at`, `expires_at`, `surcharges jsonb`, `quote_id`. A bid is never updated in amount: a carrier who changes their price creates a new bid and the previous one goes to `withdrawn`. This makes the bid history exact, which the pricing experiments need (see [[pricing-experiment-guidelines]]).

`loads.bidding_state`: `open`, `closed_accepted`, `closed_expired`, `closed_cancelled`. `loads.bidding_closes_at` is set at posting (default 4 hours for same-day loads, 24 hours otherwise, shipper-configurable between 30 minutes and 72 hours).

## Closing rules

- First accept wins: the shipper accepts one bid, `bid-svc` marks it `accepted`, every other open bid `declined`, and emits `LoadAwarded`. All in one transaction with `SELECT FOR UPDATE` on the load row, so two accepts on the same load are serialised and the second gets `409 load.already_awarded`.
- Expiry: the job `ExpireBidding` runs every minute and closes loads past `bidding_closes_at` with no accept. Bids go to `expired`. Details and the auto-decline rule in [[bid-expiry-and-auto-decline]].
- A shipper can re-open an expired load once, which resets `bidding_closes_at` and notifies carriers who bid before; 31 % of re-opened loads get awarded.

## Quote at bid time

When a carrier opens the bid form, the app calls `GET /api/v1/loads/{id}/quote`, which returns the suggested price, the breakdown of surcharges ([[surcharge-rules-catalog]]) and the shipper cap if any ([[shipper-price-cap-rule]]). The `quote_id` is stored with the bid so that we know which suggestion the carrier saw. About 44 % of bids are within 5 % of the suggestion.

## Volumes

May 2026: 118 000 loads posted, 6.1 bids per load median, 71 % awarded, median time to first bid 11 minutes, p90 68 minutes. `bid-svc` handles 40 rps average, 300 rps at the 08:00 to 09:00 peak.
