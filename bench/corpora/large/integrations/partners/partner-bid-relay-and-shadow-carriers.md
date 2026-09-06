---
name: partner-bid-relay-and-shadow-carriers
description: How a partner carrier's bid becomes a Halden bid: shadow carrier orgs, matching by registration number, driver account at acceptance, 90-day purge
type: reference
status: active
verified: 2026-06-20
---

# Bids from partner carriers and shadow carrier orgs

A carrier on Cargolink or Fretzone who bids on our listing is not a Halden user. To show the bid to our shipper, and to run the load on Halden if accepted, we need a carrier org on our side. That is a shadow org.

## Creating a shadow org

On the first inbound bid from a partner carrier id we have not seen, `ShadowCarrierProvisioner` creates an organisation with:

- `is_shadow = true`, `shadow_partner = 'cargolink'`, `shadow_external_id = <their carrier id>`

- name, country, and the partner's rating copied from the partner's carrier endpoint (`GET /v3/carriers/{id}` for Cargolink, included in the pull item for Fretzone)

- `kyc_state = 'PARTNER_VERIFIED'`: we rely on the partner's verification, per contract. No Verifid case. Payouts to a shadow carrier go through the partner, not through Payla, so KYC on our side is not needed for money.

- `no_solicitation_until = now() + 12 months` for Fretzone ([[partner-fretzone-contract-quirks]]), null for Cargolink.

- no users, no API keys, no drivers until acceptance.

## Matching to an existing Halden carrier

Before creating, the provisioner checks whether the partner carrier is already a Halden carrier: same country and same company registration number (the partner sends it for Cargolink; Fretzone sends the SIREN for French carriers, nothing for others). A match links the bid to the **real** org, sets `partner_links` on that org, and no shadow is created. The shipper sees a normal Halden carrier with a "also via Cargolink" tag.

This matters for the commission: Cargolink's contract exempts carriers already active on Halden in the previous 90 days ([[partner-cargolink-overview]]). The reconciliation uses the link and the org's activity to compute it.

Match rate in June 2026: 31 % of Cargolink bidders were existing Halden carriers, 12 % for Fretzone (fewer registration numbers).

## What the shipper sees

A bid card like any other, with the partner's logo, the partner's rating (labelled "Cargolink rating", never mixed with ours), the carrier name, and no contact details. Accepting works as usual. The shipper cannot message a shadow carrier before acceptance; after acceptance, the usual channels open and the driver app carries the rest.

## At acceptance

- Cargolink: we call `listings/{id}/status` with `awarded` and their carrier id; they notify the carrier. Fretzone: we push the attribution in the feed.

- A driver account is created on the shadow org from the phone number the partner provides with the bid (mandatory in both partners' bid payloads since we asked in 2025-11). The driver gets the standard login SMS and uses the Halden app for pickup, tracking and POD. About 70 % log in within an hour of acceptance; the rest generate `driver:login` tickets that support handles with the normal playbook.

- The shadow org's admin contact (e-mail from the partner) receives a summary e-mail with the load details and a link to the carrier back-office in read-only mode, no account creation required (a signed link valid 30 days). They can see the load, the driver, the POD. They cannot bid on other loads from there; that would bypass the partner.

## What a shadow org cannot do

Bid directly on Halden (no users), be found in the carrier directory, be exported, be invoiced by us (the partner invoices them), be rated by our algorithm (we display the partner's rating). If a shadow carrier wants to become a Halden carrier, they sign up normally; the sign-up matches the registration number and merges the shadow's history into the new org (HF-3140-bis, shipped May 2026), which has happened 23 times, and is the growth channel the sales team likes ([[partner-feedback-sales-wants-more-boards]]).

## Purge

A shadow org with no load activity for 90 days is anonymised and disabled by `app:partners:shadow:purge` (nightly): name replaced by `Partner carrier (purged)`, external id kept for re-matching, rating and contact dropped. Required by both contracts. About 400 purges a month.

## Data platform

Shadow orgs are excluded from the warehouse's carrier dimension by the `is_shadow` flag at ingestion; loads dispatched to them appear with `carrier_kind = 'partner'` and the partner name, nothing more.

## Edge cases we handle

- **Same carrier on both partners.** A German carrier active on Cargolink and Fretzone bids on our listing from both. Two shadow orgs would be wrong. The provisioner matches on registration number across partners too, and one shadow org can carry two `shadow_external_ids` (a jsonb map by partner since HF-3142). Before that, 40 duplicate shadows existed; merged by a one-off script.

- **Shadow carrier bids twice on the same load** (from the partner's UI, allowed by them). Our unique constraint on `(load_id, carrier_id)` for open bids rejects the second; we answer the callback with 409 `bid_exists` and Cargolink shows "already bid" to their carrier. Fretzone's pull just skips it.

- **Shipper favourites a shadow carrier.** Allowed; the shadow org then sees the shipper's `PRIVATE` loads on the partner board too, because the sync pushes `PRIVATE` loads to partners only for favourited shadow carriers (a per-listing audience field both partners support). Twelve shippers have done this; it is the "we like this carrier, keep them" path before they sign up.

- **Shadow carrier disputes a rating.** They have no rating on our side. Their partner rating is what shippers see, and a complaint about it goes to the partner. Support has a macro that says so politely.

- **Driver of a shadow carrier asks for GDPR erasure.** Same procedure as any driver; the driver account is ours. The partner is not involved, except that the phone number came from them and we tell them we erased it on our side so their next bid does not recreate it (they send it again; we create a new driver row; that is acceptable and documented).

## Figures, June 2026

3 900 shadow orgs alive (not yet purged), 1 100 of them with at least one dispatched load, 320 drivers logged in from shadow orgs in the month, 23 conversions to full Halden carriers since the merge flow exists, 400 purges a month. Commission-exempt bids (existing relationship) were 31 % of Cargolink acceptances.
