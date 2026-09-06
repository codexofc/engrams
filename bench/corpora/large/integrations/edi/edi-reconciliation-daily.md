---
name: edi-reconciliation-daily
description: edi:reconcile compares each partner's daily count file with edi_messages and load states, classifies gaps into 6 kinds and reports at 07:30; 0.4 % gap rate
type: reference
status: active
verified: 2026-04-30
---

# Daily EDI reconciliation

Large shippers count. At month end their transport controlling compares the loads they instructed with the loads we invoiced, and any gap becomes a dispute that costs a week of emails. The daily reconciliation exists so that the month-end comparison is boring.

## Inputs

Per partner, one of two things:

1. **A daily count file** from the partner's TMS: a flat file (CSV or a small EDIFACT `CONTRL`-like summary, depending on the partner) listing every reference they sent us the previous day with timestamp and function code (original, replacement, cancellation). Nordkarton, Bruma Retail and Kalmarine send one, over the same AS2 channel ([[as2-transport-setup]]), between 04:00 and 06:00.

2. **Our own inbound log** compared against **their acknowledgements of our outbound**: for Vestaflor and Steelhaven, who do not produce a count file, we take our `edi_messages` rows for the day and check that every IFTSTA we sent got a CONTRL back. This catches transport failures but not "they sent something we never received", which the count file would; those two partners accept the gap and check on their side weekly.

Our side: `edi_messages (id, partner_id, direction, type, message_ref, partner_reference, received_at | sent_at, status, load_id, raw_key)`, one row per message either way, and the `loads` table for state.

## The job

`edi:reconcile --date 2026-04-29` runs at 07:00, per partner, and classifies every difference into one of six kinds:

| Kind | Meaning | Typical cause | Action |
|---|---|---|---|
| `missing_inbound` | in their count file, not in `edi_messages` | AS2 delivery failed and their retry gave up; or their file lists a message they queued but never sent | ask partner to resend, or check AS2 MDN log |
| `unexpected_inbound` | in `edi_messages`, not in their file | their file was generated before the message was sent (timing), or a test message from their staging | wait one day; if still there, ask |
| `rejected_open` | we rejected, reject still open in the queue ([[edi-rejects-handling]]) | mapping data | support resolves |
| `state_mismatch` | their file says cancelled, our load is dispatched (or vice versa) | cancellation IFTMIN arrived after dispatch and was rejected `load_already_dispatched` | commercial call; the load is real for us |
| `status_unacked` | our IFTSTA got no CONTRL in 24 h | their AS2 endpoint down, or they silently dropped it | resend once, then ask |
| `duplicate_detected` | two originals with the same reference | see [[incident-2026-03-edi-duplicate-loads]] | should be zero since the fix; alert if not |

Gap rate (any kind, over messages of the day) in April 2026: 0.4 %. Almost all `unexpected_inbound` timing gaps that disappear the next day, and `rejected_open`.

## The report

Posted at 07:30 to `#edi-ops` and stored in `edi_reconciliation_runs (date, partner_id, counts jsonb, report_md)`: one block per partner, counts per kind, and the list of references for anything not `unexpected_inbound`. The person on integrations duty reads it with coffee; on a normal day it says "5 partners, 0 to act on".

Two things escalate automatically: `duplicate_detected` above zero pages the integrations rota (it means the idempotency guard failed), and `missing_inbound` above 5 for one partner opens a ticket assigned to the account's commercial owner with the partner's EDI contact in copy.

## Month end

`edi:reconcile --month 2026-03 --partner nordkarton` aggregates the daily runs and adds the invoicing dimension: every load created from their IFTMIN that reached `delivered` must have an invoice line (the billing project owns invoicing; we only check the join), and every INVOIC we sent ([[edi-invoice-invoic-outbound]]) must have a CONTRL. The output is a two-page PDF that the commercial owner sends to the partner's controlling **before** they ask. Since we started sending it (January 2026), Nordkarton's month-end dispute emails went from about 15 loads a month to 2.

## What the reconciliation does not do

- It does not fix anything. It classifies and points. Every fix is a human action in the reject queue or a message to the partner, because the causes are mostly on the partner's side and guessing was wrong too often in 2025.

- It does not reconcile prices. That is the billing reconciliation's job; ours stops at "the load exists and has the state both sides think it has".

- It does not run for the JSON partners ([[edi-api-json-alternative]]): their API calls are synchronous and the response is the acknowledgement, so the daily gap problem does not exist for them.

## A report on a bad day

For the record, the 2026-03-11 report for Bruma Retail (the duplicate loads incident), which is the only time `duplicate_detected` has been non-zero:

```
partner: bruma            date: 2026-03-11
inbound in count file:    218      inbound in edi_messages: 430
missing_inbound:          0
unexpected_inbound:       212   (references matching ^BR-.*-1$)
rejected_open:            3
state_mismatch:           0
status_unacked:           2
duplicate_detected:       212   ** PAGED **
```

The page fired at 07:30; the incident had already been opened at 08:52 by the customer call because the on-call had read the page as "reconciliation noise after a partner outage" and planned to look after the morning standup. The lesson went into the runbook line for that alert: `duplicate_detected` is never noise, and it means loads are on the market right now.

Since that day the report also prints the age of the oldest unresolved line per partner, so that a `rejected_open` from Monday is still visible on Thursday rather than buried under Thursday's fresh counts.
