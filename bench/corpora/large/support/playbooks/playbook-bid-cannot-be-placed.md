---
name: playbook-bid-cannot-be-placed
description: Carrier cannot bid: the 409 codes from POST /v2/bids (carrier_not_verified, insurance_expired, load_not_open, own_load) and the 429 case
type: reference
status: active
verified: 2026-06-11
---

# Carrier cannot place a bid

Category `load:*` (sub-tag `bid`). The carrier clicks "Bid", the web app shows an error. The API answers 409 with a code in the error envelope; the web app translates the code into a sentence. Ask for the exact sentence, or better, the `request_id` shown under the error.

## Codes

`hfctl org get <carrier_org_id>` and `hfctl load get <load_id>` answer most of them.

- `carrier_not_verified`: KYC not `VERIFIED`. Web text "Your company must be verified to bid". See [[playbook-kyc-verifid-stuck]].

- `insurance_expired`: the carrier's goods-in-transit insurance certificate in `documents` has `valid_until` in the past. Web text "Your insurance certificate has expired". The carrier uploads a new one in Settings, Documents; it is accepted immediately, no review. Macro `bid-insurance-expired`.

- `insurance_below_load_value`: the load declares a goods value above the certificate's cover. Web text mentions the two amounts. The carrier cannot bid on this load unless they upload a higher cover. Not a bug, macro `bid-insurance-cover`.

- `load_not_open`: the load is no longer `OPEN` or `BIDDING`. The search results can be up to a minute behind ([[playbook-load-not-visible-in-search]]), so a load that was just dispatched still shows. Macro `bid-load-taken`.

- `bid_below_floor`: the shipper set a minimum, the bid is under it. Shown in the form since HF-3130, so this one is rare now.

- `own_load`: the org is both shipper and carrier (a few logistics companies are) and tries to bid on its own load. Refused on purpose.

- `vehicle_type_mismatch`: the load requires a vehicle type (`FRIGO`, `ADR`, `MEGA`) the carrier has not declared in its fleet. They add the vehicle type in Settings, Fleet.

- `bid_window_closed`: the load's `bidding_closes_at` has passed. The shipper can extend it, we cannot.

## Rate limit

A 429 instead of a 409: the carrier is over the per-org limit. Happens with integrators that place bids by API, almost never from the web. See [[playbook-integrator-rate-limited-429]].

## Nothing happens on click

No error, no bid: check the browser. Two known cases: an ad blocker that blocks the `/v2/bids` call because the URL contains "bid" (yes), and a very old cached version of the web app after a release. Macro `web-hard-refresh` and the ad blocker paragraph.

## Verify a bid went through

`hfctl load get <load_id>` lists bids with `carrier_id`, `amount`, `status`. If the bid is there, the carrier's screen was not refreshed. If it is there twice, escalate, that should not happen (the unique constraint on `(load_id, carrier_id)` for open bids).

## Do not

We do not bid on behalf of a carrier, ever, even with their written consent. A bid is a commitment.
