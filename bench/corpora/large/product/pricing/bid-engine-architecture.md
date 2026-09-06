---
name: bid-engine-architecture
description: bid-svc owns immutable bids with a state column, first-accept or expiry closes a load, pricing-svc quotes over gRPC in 150 ms
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

## Concurrency and idempotency

Bid creation is `POST /api/v1/loads/{id}/bids` with an `Idempotency-Key` header, mandatory since HF-2205 after the mobile app retried a bid on a flaky connection and a carrier had three identical open bids. The key is stored in `bid_idempotency_keys` for 24 hours; a repeat returns the original bid with `200` instead of `201`.

A carrier can hold at most one open bid per load. Creating a second one withdraws the first in the same transaction (`WITHDRAWN` with `withdraw_reason = 'replaced'`), which is how the "change my price" flow is implemented client-side: it is just a new bid. The uniqueness is a partial index `bids_one_open_per_carrier_load ON bids (load_id, carrier_id) WHERE state = 'open'`, so a race between two replacements ends with one `409 bid.concurrent_replace` and the client retries.

The accept path takes the load row lock first, then reads the bid; a bid withdrawn between the shipper's click and the accept returns `409 bid.no_longer_open` with the current top bid in the body so the UI can offer it.

## Events emitted

`bid-svc` publishes on the internal bus:

- `BidPlaced`, `BidWithdrawn`, `BidDeclined` (with reason), `BidExpired`, `BidAccepted`
- `LoadAwarded` (consumed by dispatch, billing's self-billing flow, the ETA service for the `at_award` prediction, and the auto-decline of [[bid-expiry-and-auto-decline]])
- `LoadBiddingClosed` with the closing reason

And the product analytics events `bid.placed`, `bid.withdrawn`, `bid.ranked` (one per bid per ranking recomputation, sampled 1 in 10 above 20 bids on a load because a busy load recomputes hundreds of times), `load.awarded`. The `bid.ranked` sampling is why the warehouse's `rank_at_close` is missing for some bids.

## Failure modes seen

- 2025-08: `pricing-svc` gRPC deadline set to 150 ms but the client library added its own 5 s connect timeout on a cold channel; the first quote after a `pricing-svc` restart took 5 s and the bid form spun. Fixed by a warm-up call at startup and `keepalive` on the channel.
- 2025-12: the lane quote cache had no jitter on its 10-minute TTL, and 40 000 cache entries expired at the same minute after a deploy, sending a burst to `pricing-svc`. TTL is now 10 minutes plus a random 0 to 120 s.
- 2026-03: `ExpireBidding` and a manual load cancellation raced on the same load; both won their transactions in sequence and the load ended `closed_expired` with `cancelled_at` set. The job now skips loads with `cancelled_at IS NOT NULL` and the state machine refuses `closed_expired` after `closed_cancelled`.

## Capacity numbers

- `bids` table: 9.1 M rows, 6 GB with indexes; partitioned by month of `created_at` since March 2026, 13 months kept online, older months archived to the warehouse only.
- Peak sustained: 300 rps on `bid-svc`, of which 60 % are `GET .../quote` and `GET .../bids` (the shipper list polling every 15 s). The list endpoint is cached 5 s per load in the process cache; a longer cache made shippers miss bids that arrived during the cache window and accept a bid that was already outbid.
