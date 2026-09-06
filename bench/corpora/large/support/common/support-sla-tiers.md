---
name: support-sla-tiers
description: Support SLAs by plan: first response 8 h, 4 h, 1 h on business hours, what stops the clock, and the measured H1 2026 figures
type: reference
status: active
verified: 2026-07-03
---

# Support SLAs by plan

The contractual figures live in the order form. This is what they mean day to day and how we measure them.

### Targets

| Plan | First response | Resolution target | Hours | Channel |
|---|---|---|---|---|
| Starter | 8 business hours | 5 business days | Mon-Fri 8:00-18:00 CET | Deskline form, e-mail |
| Business | 4 business hours | 2 business days | Mon-Fri 8:00-18:00 CET | same plus chat |
| Enterprise | 1 hour | 1 business day, 4 h for blocking | Mon-Sat 7:00-20:00 CET | same plus shared channel and phone |

"Blocking" for Enterprise means a load `IN_TRANSIT` or `DISPATCHED` that cannot progress, an invoice run that cannot close, or an integration that receives nothing. A cosmetic bug is never blocking whatever the customer says, and the account manager is the one who says it to them.

## What the clock measures

The clock starts when the ticket is created in Deskline, whatever the channel. It stops at the first human answer that is not the auto-acknowledgement and not a `need-reference` macro asking for identifiers. This last point was argued for a while: a macro asking for a load id is not a response, it is us admitting we cannot start. Since HF-3002 the macro does not stop the clock.

Resolution stops when the ticket is set to `solved`. Reopening within 7 days restarts it from the original creation time, so a premature `solved` costs us.

Time outside business hours does not count for Starter and Business. Enterprise "1 hour" is on their extended window. Public holidays follow the French calendar for everyone, which annoys the German customers twice a year and is written in the order form.

## Breach handling

Deskline flags breaches automatically. A breach on Enterprise triggers a message to the account manager and a line in the monthly review. Three breaches in a quarter on one Enterprise account and the contract says we owe a 5 % credit on the support fee, which happened once (Q4 2025, a shipper in the metals sector, after the [[support-escalation-levels-l1-l2-l3]] was not followed for a webhook outage).

## Measured, H1 2026

From the Deskline export in the warehouse (`marts.support_tickets`, computed weekly):

- Starter first response median 2 h 10, p95 7 h 40, breaches 3.1 %

- Business first response median 55 min, p95 3 h 20, breaches 1.8 %

- Enterprise first response median 14 min, p95 48 min, breaches 0.6 % (11 tickets over the half)

- Resolution: 71 % of tickets solved at first contact, 19 % after one L2 pass, 10 % needed a backend ticket

The p95 on Starter is close to the target on Mondays because of the weekend backlog. Discussed in the triage ([[support-weekly-triage-ritual]]), decision: no weekend coverage for Starter, we accept the Monday number.

## What we do not promise

No SLA on feature requests, on partner marketplaces we do not operate (Cargolink or Fretzone being down is not our breach, but we do tell the customer), and no SLA on the driver app when the phone is not on the supported list.
