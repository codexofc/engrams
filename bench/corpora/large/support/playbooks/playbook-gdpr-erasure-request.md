---
name: playbook-gdpr-erasure-request
description: GDPR erasure request: identity check, 30-day clock, blockers (active load), what hfctl gdpr erase removes and what legally stays
type: project
status: active
verified: 2026-04-16
---

# GDPR erasure request

Category `other:gdpr`. A driver (most cases), a shipper user, or a carrier admin asks that their personal data be deleted. We have 30 days to answer. The technical part takes an hour, the identity check takes the time it takes.

## Who can ask

A natural person, about their own data. A carrier asking us to erase a former driver is not an erasure request from the data subject; we treat it as an account disable (the carrier can do it) and tell them that the driver can ask for erasure themselves. A shipper company asking to delete "all their data" at contract end is offboarding, a different procedure with the account manager.

## Identity check

The request must come from the e-mail or phone number on the account, or be accompanied by a copy of an ID that matches the name on the account (deleted from the ticket once checked, we do not keep it). For drivers, who log in by phone, the SMS link `hfctl driver verify-identity <driver_id> --apply` sends a one-time code they must give back in the ticket. No check, no erasure.

## Blockers

`hfctl gdpr check <subject_type> <subject_id>` lists what prevents erasure:

- an active load (the driver is assigned to a `DISPATCHED` or `IN_TRANSIT` load): wait until it terminates, tell the requester.

- an open damage claim or a legal hold flag on the org: legal team decides, the 30 days may be extended to 90 with a written reason to the requester.

- a payout in progress for a sole-trader carrier whose driver is the owner: wait for `SETTLED`.

## What erasure does

`hfctl gdpr erase <subject_type> <subject_id> --ticket <deskline id> --apply` (L2), in one job:

- name, e-mail, phone, photo, device list, PIN hash and tokens replaced by placeholders (`erased-<short id>`); the row is kept so that foreign keys hold.

- GPS positions attributed to the driver are deleted after 30 days anyway (retention rule), the job deletes what remains.

- free-text fields the person wrote (bid comments, chat messages) are replaced by `[erased]`.

- the warehouse receives an erasure event and its own pipeline removes the person from the raw and core tables within 7 days.

- Deskline tickets opened by the person are anonymised by the Deskline admin (a manual step, in the checklist).

## What it does not touch

Invoices, credit notes, delivery records (`load_events`, PODs) and audit logs keep their content. These carry a legal retention (10 years for invoices in most of our countries) and the name of a driver on a POD is part of a commercial document. The confirmation letter says so explicitly, with the legal basis; the macro `gdpr-erasure-done` has the paragraph in four languages. Nobody has contested it so far.

## Timeline

Day 0 ticket, identity check within 5 business days, erasure within 20 days when no blocker, letter on the day of erasure. A blocker extends the clock; we tell the person within the first 30 days what blocks and when we expect to proceed.

## Figures

31 requests in the 12 months to April 2026, 27 drivers, 3 shipper users, 1 carrier admin. 4 blocked by active loads for one to nine days. None refused. Median time to erasure 9 days, driven by the identity check.

## Escalate

Legal for anything with a hold or a claim. Backend if `gdpr erase` fails half way; it is idempotent and can be re-run, but they want to see the error.
