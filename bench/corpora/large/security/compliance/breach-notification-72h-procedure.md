---
name: breach-notification-72h-procedure
description: A personal data breach opens breach_assessments within 4 h, gets a DPO risk decision within 24 h, authority notice within 72 h, controllers within 48 h
type: reference
status: active
verified: 2026-03-05
---

# 72 hour breach notification procedure

This is the compliance half of a security incident. The incident itself is run by the incidents project's process; this procedure starts when the incident commander says the words "personal data may be affected", and it runs alongside.

## Clock

The 72 hours to the supervisory authority start when we become **aware** of the breach, which we define as the moment an on-call engineer confirms that personal data was accessed, disclosed, altered or lost, not the moment an alert fires. The confirmation time is written in `breach_assessments.aware_at` and everything counts from there. If in doubt, the earlier time.

## Steps

**Within 4 h of awareness: open the assessment.** `bin/console compliance:breach:open --incident HF-xxxx` creates `breach_assessments (id, incident_ticket, aware_at, opened_by, categories, subjects_estimate, status)`. The person on the compliance rota fills the categories from the data map ([[gdpr-data-map]]) and a first estimate of how many people. Rough is fine; "between 100 and 10 000 drivers, location category" is a valid first estimate.

**Within 24 h: DPO risk decision.** The DPO (external, reachable by phone, number in the on-call sheet) receives the assessment and the incident timeline so far. Three outcomes, recorded in `status`:

- `no_risk`: documented, not notified. Example: encrypted backup medium lost with the key safe elsewhere. Documentation still goes in the register of breaches.

- `risk`: notify the authority, not necessarily the individuals.

- `high_risk`: notify the authority and the individuals (and their controller when we are processor).

**Within 72 h: authority.** The notification is filed through the national authority's online form by the DPO or the head of engineering. The draft is written from `templates/breach-notification.md` in `halden-legal`, in the local language. If facts are still missing at 72 h we file what we have and mark it as a phased notification; we do not wait to be complete.

**Within 48 h of the decision: customers and individuals.** When we are processor for the affected data (drivers' data uploaded by carriers, EDI contact persons), the controller is the carrier or shipper and our DPA promises them 48 h (24 h for the variants, see [[dpa-carriers-template]]). Template `templates/breach-controller-notice.md`. When we are controller and the risk is high, individuals are notified by email from `privacy@halden.example` with what happened, what data, what we did, what they can do.

## Register of breaches

Every assessment, including `no_risk`, stays in `breach_assessments` forever (no retention rule; the volume is tiny). Five rows in March 2026: two `no_risk` (a misaddressed invoice email, a test dataset with 12 real emails found in staging), two `risk` (the incidents project has both post-mortems), one `high_risk` (the public bucket incident of February 2026, 2 300 drivers' POD photos with signatures; authority notified at 51 h, carriers at 30 h, individuals through carriers).

## What the drills taught

A tabletop in March 2026 with a made-up scenario (a support laptop stolen with an open session) showed:

- Nobody knew who had the authority portal credentials. Now two people, listed in the on-call sheet, credentials in the vault.

- The estimate of affected people took 3 hours because it needed a query nobody had written. `compliance:breach:estimate --category documents --since <date>` now exists and answers the common shapes in under a minute.

- The DPO was reachable but the incident channel had not invited them. The channel template now includes the DPO's account from the start.

## Rules

- The 72 h is not a target, it is a limit. The real target is 48 h.

- Never notify individuals before the controller when we are processor: the carrier tells their drivers, we give them the text.

- Never say "no data was accessed" until the logs have been read by two people. "We have no evidence of access" is the honest sentence until then, and the templates use it.
