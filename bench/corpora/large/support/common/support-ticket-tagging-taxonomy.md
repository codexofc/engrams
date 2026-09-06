---
name: support-ticket-tagging-taxonomy
description: Deskline ticket categories fixed in HF-3005 (12 closed values), the one-category rule and the H1 2026 split by category
type: project
status: active
verified: 2026-07-10
---

# Ticket categories

Before HF-3005 (November 2025) tags were free text and we had `login`, `Login`, `driver login`, `connexion chauffeur` and `pin` for the same thing. The weekly triage could not answer "what do we get most". Now the `category` field is a closed list and sub-tags use a colon.

### Categories

| Category | Covers | Typical playbook |
|---|---|---|
| `load:stuck` | a load that does not move to the next state | the stuck-load playbook (support/playbooks) |
| `load:cancel` | cancellation requests and refusals | cancellation playbooks |
| `load:duplicate` | a load published twice, or twice via a partner | duplicate playbooks |
| `load:search` | a carrier cannot find a load, or sees a wrong one | search playbook |
| `driver:login` | PIN, phone number, locked account, second device | driver login playbook |
| `driver:sync` | app not syncing, POD not uploaded, GPS not moving | sync and GPS playbooks |
| `invoice:dispute` | amount, VAT, currency, missing lines | invoice dispute playbook |
| `invoice:delivery` | invoice e-mail not received, wrong recipient | invoice e-mail playbook |
| `payment:payout` | carrier payout late or missing | payout playbook |
| `account:kyc` | Verifid pending or rejected | KYC playbook |
| `integration:webhook` | integrator receives nothing or duplicates | webhook playbook |
| `integration:partner` | Cargolink or Fretzone mismatch | partner sync playbook |
| `other` | must carry a free-text sub-tag | none |

`other` above 8 % of a week triggers a review in the triage: either a category is missing or L1 is lazy.

## Rules

- One category per ticket. If a driver cannot log in because the carrier's KYC was rejected, the category is `account:kyc`, because that is what we have to fix. The symptom goes in the title.

- The category is set by L1 at first touch, never changed by L2 unless L1 was wrong, and then L2 says so in an internal note so the triage sees it.

- `incident:<id>` is a separate tag, not a category. A ticket keeps its category during an incident.

## Split, H1 2026 (7 412 tickets)

From `marts.support_tickets`:

- `load:stuck` 21 %

- `driver:login` 25 % (of which 60 % "forgot PIN", 25 % "second device", the rest disabled accounts)

- `driver:sync` 11 %

- `invoice:*` 14 %

- `payment:payout` 6 %

- `account:kyc` 5 %

- `integration:*` 8 %

- `load:*` other 6 %

- `other` 4 %

The `driver:login` share is why the self-service PIN reset by SMS link was pushed up in the mobile roadmap. The `load:stuck` share is almost entirely `DISPATCHED` loads whose driver never pressed pickup, see the playbook. These two categories are what the [[support-weekly-triage-ritual]] looks at first.

## Export

Deskline exports nightly to `raw.deskline_tickets` (warehouse), the category field is `custom_fields.category`. The mart splits the colon into `category_group` and `category_detail`.
