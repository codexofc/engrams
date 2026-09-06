---
name: bid-expiry-and-auto-decline
description: Bids expire with the load window via ExpireBidding every minute, and a carrier's overlapping bids are auto-declined on award
type: project
status: active
verified: 2026-02-18
---

## Expiry

Every load has `bidding_closes_at` (see [[bid-engine-architecture]]). `ExpireBidding` runs every minute, selects loads with `bidding_state = 'open' AND bidding_closes_at < now() - interval '90 seconds'` and closes them. The 90 seconds absorb clock skew between the app servers and a last-second accept by the shipper, which is committed with the load row locked, so the job cannot close a load in the middle of an accept.

A closed load's open bids go to `expired`. Carriers are not notified of expiry per bid (too noisy, a carrier with 30 open bids a day would get 30 emails); they see it in the app under "bids > expired" and get one daily digest at 18:00 if they want it (`carrier_settings.expiry_digest`).

Shippers get a push at closing: "Your load X closed with N bids" or "Your load X closed without bids, re-open?". Re-opening is allowed once, extends by the original window, and pings the carriers who bid before.

## Auto-decline on overlap

A carrier can bid on several loads for the same truck. When one is awarded, the others become impossible, and until October 2025 the shipper of the other load could accept a bid whose carrier was already committed, then discover it at pickup time. This was 30 % of the no-shows.

Since HF-2130, on `LoadAwarded`, `bid-svc` looks at the carrier's other open bids and auto-declines those whose pickup window overlaps with the awarded load's pickup-to-delivery span plus 4 hours of positioning, unless the carrier has declared more than one available truck (`carrier_accounts.declared_trucks > 1`, then the threshold is the number of trucks). The declined bids carry `decline_reason = 'carrier_committed'` and the carrier gets one notification listing them.

Measured effect: no-show rate from 2.9 % to 2.1 % in the following quarter; 11 % of bids are now auto-declined, of which 3 % are contested by the carrier ("I had a second truck"), which is why the declared-trucks threshold exists.

## Withdrawal

A carrier can withdraw an open bid at any time until it is accepted. A bid withdrawn less than 15 minutes after being top-ranked in the shipper's list counts against reliability (the "flaky bid" component) because it is usually a carrier fishing for the price. See [[bid-ranking-score]] for how reliability enters the ranking.

## Timings that matter

- Default windows: 4 hours for same-day pickups, 24 hours otherwise.
- Shipper-configurable: 30 minutes to 72 hours.
- Median time to award after closing: 22 minutes (shippers get the push and pick).
- 8 % of loads are awarded in the last 10 minutes of the window, so a job that closed early would lose real awards; that is why the tolerance is on the late side.
