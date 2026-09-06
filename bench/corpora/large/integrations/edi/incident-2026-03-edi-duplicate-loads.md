---
name: incident-2026-03-edi-duplicate-loads
description: On 2026-03-11 Bruma Retail re-sent 212 IFTMIN with references suffixed -1; 212 duplicate loads published, 9 got bids; fixed by a fuzzy guard and a 30 min hold
type: project
status: active
verified: 2026-04-08
---

# Duplicate loads from a partner re-send (March 2026)

Ticket HF-2135. Integration incident with a commercial consequence: carriers bid on loads that did not exist twice. Handled under the platform on-call; written here because both the cause and the fix are in the gateway.

## What happened

- **2026-03-11 02:00 to 05:30 UTC**: Bruma Retail's TMS had an outage. Their EDI queue kept the 212 IFTMIN messages they had generated on the evening of 03-10, which we had **already received and processed** before the outage (we had sent APERAK positive for all of them, and their count file of 03-11 morning listed them once).

- **05:45**: their recovery procedure re-generated the queued messages. Their TMS, to avoid what it considered a duplicate on its own side, appended `-1` to the shipment reference in `BGM.1004` (`BR-2026-77812` became `BR-2026-77812-1`) and issued new AS2 `Message-ID`s. Function code stayed `9` (original).

- **05:45 to 05:52**: 212 IFTMIN pass both our guards: transport-level `Message-ID` is new ([[as2-transport-setup]]), application-level `partner_reference` is new because of the suffix ([[edifact-iftmin-mapping]]). 212 loads created and **published to the marketplace immediately**, as EDI loads were, with the same pickup and delivery as 212 existing loads.

- **06:10 to 08:40**: carriers see pairs of identical loads. 9 of the duplicates receive bids; 2 bids are auto-accepted at target price (the platform's auto-accept feature).

- **08:40**: a Bruma dispatcher, looking at their web view, sees 424 loads instead of 212 and calls support. Incident opened 08:52.

- **09:05**: the 212 `-1` loads identified by `partner_reference LIKE '%-1'` and `created_at` in the window; cross-checked against the 03-10 originals by pickup site, delivery site and slot: 212 exact matches.

- **09:20**: the 203 duplicates without bids cancelled with reason `duplicate_edi`. The 7 with open bids: bids rejected with a message to the carrier, then cancelled. The 2 auto-accepted: carriers called by support; both agreed to cancel (one asked for and got a goodwill payment because they had already assigned a driver).

- **10:30**: APERAK negative sent for the 212 `-1` references with reason `duplicate_reference` so Bruma's TMS marks them as rejected. Incident closed.

## Root cause

Our idempotency key was exactly the partner's reference, and the partner's recovery procedure changed the reference. Two guards, both keyed on values the sender controls, both bypassed by the same sender behaviour. Contributing: EDI loads were published to carriers the moment they were created, so a duplicate was visible to the market within seconds.

## Fixes

- **Fuzzy reference guard** (HF-2139, deployed 2026-03-13): for each partner, `IftminToLoadMapper` normalises the reference with a per-partner regex (`edi_partner_overrides` key `reference_normalize`, for Bruma `^(.*?)(-\d)?$` keeping group 1) before the uniqueness check. A normalised match with an existing load of the same partner in the last 30 days is a `duplicate_reference` reject. Three other partners' TMS documentation was checked for similar re-send conventions; Kalmarine appends `/R`, now covered.

- **Content fingerprint**: independent of the reference, a load whose `(partner_id, pickup_site, delivery_site, pickup_slot_from, pallet_count)` equals an existing non-cancelled load of the same partner created in the last 7 days is accepted with a **warning** (not rejected, because two genuine loads a week apart with the same shape do happen) and flagged `possible_duplicate` in the back-office EDI view. 3 flags a week since; 1 was real.

- **30 minute publication hold** for EDI-created loads (HF-2142): a load from IFTMIN is created in `PENDING_PUBLICATION` and published 30 minutes later unless cancelled or flagged. Bruma's dispatcher would have seen 424 loads in the web view during the hold and none of them would have been on the market. Cost: 30 minutes of market time for 3 100 loads a month; the commercial team accepted it, and two partners asked for the hold to be configurable to 0 for their urgent loads, which it now is per partner (`publication_hold_min`), default 30.

- **Reconciliation kind `duplicate_detected`** now also fires on the content fingerprint, not only on exact reference; see [[edi-reconciliation-daily]].

## What we asked the partner

To change their recovery procedure to re-send with function code `5` (replacement) and the original reference, which our mapper handles as an update or rejects as `load_already_dispatched`. Their answer, 5 weeks later: done in their next TMS release, scheduled for Q3 2026. The fixes above do not depend on it.

## What it cost

- Carrier side: 9 carriers bid on loads that vanished. Two had assigned drivers. One goodwill payment of 150 EUR. Three carriers mentioned it in the next satisfaction survey.

- Shipper side: Bruma's dispatchers spent about two hours checking that no real load had been cancelled by mistake. None had, because the cancellation script matched on the `-1` suffix and a full content match, and refused to touch anything else. One of their loads that genuinely had a `-1` reference from a legitimate earlier re-send convention was flagged and reviewed by hand; it was kept.

- Our side: 6 person-hours on the incident, about 4 days of work for the three fixes, and a month of watching the `possible_duplicate` flags to tune the fingerprint (the pallet count was added to the fingerprint after a week because pickup and delivery site plus slot alone flagged 12 legitimate pairs).

## Rules of thumb that came out of it

- An idempotency key controlled entirely by the sender is a courtesy, not a guarantee. Keep it, and add a guard based on content.

- Anything created by an automated inbound channel should not be visible to third parties for a few minutes. The 30 minute hold costs nothing that anyone has measured and would have turned this from an incident into a support ticket.

- When a partner's recovery procedure is documented, read it before go-live. Bruma's was in their implementation guide, page 31. Nobody had read page 31.
