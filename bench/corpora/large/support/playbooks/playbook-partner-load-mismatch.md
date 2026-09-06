---
name: playbook-partner-load-mismatch
description: Load differs between Halden and Cargolink or Fretzone: partner refs and sync runs, field ownership, resync, when to call the partner
type: reference
status: active
verified: 2026-07-08
---

# Load differs between us and a partner board

Category `integration:partner`. A shipper says the price on Fretzone is not what they set here, a carrier found a load on Cargolink that does not exist here, or a load cancelled here is still shown on the partner. The sync is one-directional per field and runs on a schedule; most tickets are timing or field ownership.

## Checks

1. `hfctl partner refs <load_id>`: for each partner, `external_id`, `last_pushed_at`, `last_pulled_at`, `sync_state` (`IN_SYNC`, `PENDING_PUSH`, `PUSH_FAILED`, `PARTNER_STALE`), `last_error`.

2. `hfctl partner runs --partner fretzone --since 24h`: the sync runs, with counts and errors. A run older than 30 minutes with none after is a stuck sync, escalate.

## By symptom

**Price differs.** Price is owned by us: what the shipper sets here is pushed to the partner within one run (Cargolink is push on event, under a minute; Fretzone is polled by them every 10 minutes). `PENDING_PUSH` older than 15 minutes or `PUSH_FAILED`: read `last_error`. A partner validation error (`price_below_minimum`, Fretzone has a floor per lane) means the partner refuses the price and shows the old one; the shipper must raise it or unpublish there. Macro `partner-price-rule`.

**Dates differ.** Same ownership, same path. Except: Fretzone shows dates in local time of the pickup, we show the shipper's timezone. A load picked up in Lisbon shown to a Paris shipper differs by one hour and that is display, not data. Macro `partner-timezone-display`.

**Load exists on the partner, not here.** Two cases. A load *published by the partner's own users* appears here only if the shipper is not also ours (dedup rule), see [[playbook-duplicate-load-published]]. A load *we pushed* and then cancelled: the cancel is pushed too; `PARTNER_STALE` means the partner acknowledged nothing. Cargolink honours cancels within a minute; Fretzone within its 10-minute poll. Beyond that, `hfctl partner resync <load_id> --partner fretzone --apply` re-pushes the current state. If it stays, the partner is at fault and we open a ticket with them (L2 has the partner support addresses).

**Load exists here, not on the partner.** `visibility` must be `PUBLIC` or `PARTNER_ONLY`, and the shipper's org must have the partner enabled (`hfctl org get`, `partners[]`). Vehicle types the partner does not know (`MEGA` on Fretzone) are not pushed at all, `last_error = unsupported_vehicle_type`. Macro `partner-not-pushed` with the reason.

**Bid from a partner carrier missing.** Bids come back from the partner as bids here with `source = cargolink`. If the carrier is not known to us, a shadow carrier org is created and the shipper sees it flagged "via Cargolink". Missing bid: `hfctl partner runs`, look for `bid_pull` errors. Escalate if the run is fine and the bid is not.

## Do not

Do not edit anything on the partner side, we have no access besides the API. Do not tell the customer the partner is "broken" in writing before L2 confirmed it, the partner reads those tickets when we forward them.

## Escalate

L2 for `resync` and for opening a ticket with the partner. Backend if a sync run has been failing for more than 30 minutes (the partners' incident notes describe what that looks like).
