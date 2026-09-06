---
name: playbook-pod-missing-or-wrong-load
description: Missing POD or POD on the wrong load: document states, phone upload queue, carrier admin upload, L2 document move keeping the sha256
type: reference
status: active
verified: 2026-06-11
---

# POD missing, or attached to the wrong load

Category `driver:sync` (sub-tag `pod`). The shipper will not pay without the proof of delivery. Either the photo never left the phone, or it landed on another load.

## Missing POD

1. `hfctl load get <load_id>`, section `documents`. Each has `kind` (`POD`, `CMR`, `DAMAGE`), `status` (`PENDING`, `AVAILABLE`, `REJECTED`), `sha256`, `uploaded_by`.

2. A `PENDING` document older than 1 hour: the app created the row, the file never arrived or the `complete` step failed. The upload resumes from where it stopped when the app has network (since 4.7). Check `hfctl driver sync-status <driver_id>`; if `outbox_depth > 0`, it is waiting. Macro `pod-upload-pending`.

3. No document at all and the driver says he took the photo: the photo is in the app's local queue. Same macro. If the driver has since deleted the app, the photo is gone. Say so plainly.

4. `REJECTED`: the file failed the size or hash check. The driver takes it again. Rare since 4.8 enforces the 25 MB cap client side.

5. The driver is unreachable or the phone is gone: since HF-3118 the carrier admin can upload a POD from the web back-office (scan, photo sent by the driver by other means). It is marked `uploaded_by = carrier_admin`. Macro `carrier-admin-upload-pod`.

## POD on the wrong load

A driver doing two deliveries in the same yard attaches the POD of load A to load B. Both shippers complain, one has a POD that is not theirs.

1. Confirm with both `hfctl load get`. Compare the document's `taken_at` (from the photo metadata, when present) with each load's `deliver` event time.

2. L2 runs `hfctl document move <document_id> --to-load <load_id> --ticket <deskline id> --apply`. The document keeps its id and `sha256`, the move is logged, and a new `document.available` webhook event is emitted for the receiving load. The old load gets `document.removed`. Integrators who dedupe on document id will see the same id with a new load id, which is intended and documented.

3. If the two loads belong to different shippers, the wrong shipper saw a document that was not theirs. This is not a data leak in the legal sense (the driver uploaded it, the document was about a delivery they were party to in the yard), but the account manager of the affected shipper is told. Decided in the [[playbook-format-feedback-too-long]] discussion's sibling triage of 2026-02.

## Shipper refuses the POD

Blurry, unsigned, wrong page. Not a support matter: the shipper asks the carrier for a better one, the carrier admin can upload it. The first document is not deleted, it gets `superseded_by` in the documents list. Macro `pod-quality-commercial`.

## Do not

We do not upload documents for carriers. We do not delete documents; they are part of the delivery record.

## Related

Load still `IN_TRANSIT` while the POD exists: [[playbook-load-stuck-in-transit-after-delivery]].
