---
name: playbook-undeliver-wrong-delivered
description: Reverting a wrong DELIVERED with hfctl load undeliver: preconditions, effects on events, POD, webhooks, rating, refusals and 2026 figures
type: project
status: active
verified: 2026-07-30
---

# Reverting a wrong `DELIVERED`

There is no `undeliver` transition in the load state machine and there will not be one soon (HF-1590 not scheduled). Until March 2026 a wrong delivery was fixed by the backend with a direct `UPDATE` and a manual `load_events` edit, about twice a month. HF-3105 turned that into `hfctl load undeliver`, an admin API endpoint that does the same thing in one transaction, with the checks and the audit trail. Support L2 runs it, the backend no longer needs to.

## When it applies

The driver pressed "Delivered" on the wrong load, or pressed it at pickup by mistake (the two buttons are far apart since 4.6, it still happens), or a carrier admin confirmed delivery from the web on the wrong line. The load is `DELIVERED`, the goods are not.

It does not apply when the goods were delivered and someone regrets it, when the shipper disputes the delivery time (that is a timeline correction, different procedure, backend only), or when the load is already `INVOICED`.

## Preconditions checked by the command

`hfctl load undeliver <load_id> --ticket HF-3xxx` (dry run) prints the checks:

- `status = DELIVERED`. Anything else is refused.

- No invoice referencing the load, draft or finalized. If a draft invoice exists, billing must delete the draft first (their runbook), then we run it. If a finalized invoice exists, we stop: the fix is a credit note, and the load stays `DELIVERED`. This is the case in about a third of requests because shippers notice at invoice time.

- The `deliver` event is less than 14 days old. Older, the command refuses and the backend decides case by case.

- The load has no open damage claim.

## What it does

In one transaction:

1. `loads.status` back to `IN_TRANSIT`.

2. The `deliver` row in `load_events` is not deleted. It gets `voided_at`, `voided_by`, `voided_ticket`. Timelines hide voided events by default; the audit view shows them struck through. A new `deliver_voided` event is appended with the ticket.

3. POD documents on the load are kept and flagged `voided = true`. They stay in the record, they are not shown to the shipper as a valid POD.

4. A webhook event `load.in_transit` is emitted with `reason = delivery_voided` and the ticket in `meta`. Integrators who model the load as a simple state field will see it go backwards; the integrator guide says states can go backwards for support corrections and gives this exact example.

5. Carrier rating: the on-time delivery figure computed from the voided event is recomputed on the next rating run. Nothing to do.

6. GPS tracking resumes on the next app sync, because the app re-reads the load state.

## Procedure

1. Deskline ticket with the shipper's or carrier's statement that the goods were not delivered, and which load the driver meant if any.

2. L2 opens the HF ticket, pastes the statement, runs the dry run, pastes the output.

3. `--apply`. Single person, no confirmation token: the action is reversible (the driver presses "Delivered" again) and no money moves.

4. Tell the carrier the driver must confirm delivery again when it really happens, and that the timestamp will be the real one. Macro `undeliver-done`.

5. If the driver meant another load, that load is still `IN_TRANSIT` or `DISPATCHED` and needs its own confirmation. Do not chain the two in one ticket, two loads, two tickets.

## Refusals and what to do instead

### By situation

| Situation | Answer |
|---|---|
| Invoice finalized | credit note through billing, load stays `DELIVERED`, [[playbook-invoice-dispute-amount]] |
| Draft invoice exists | ask billing to drop the draft, then undeliver |
| `deliver` older than 14 days | HF ticket to backend, they decide |
| Damage claim open | close or transfer the claim first, claims team |
| Load is `INVOICED` | same as invoice finalized |
| Shipper disputes only the time | timeline correction, backend only, not this command |

## Figures

From `sys_audit_log` since the command exists (2026-03-10 to 2026-07-30): 41 runs, 38 applied, 3 refused at dry run for finalized invoices (the agent ran the dry run to get the message for the ticket, fine). Median time from Deskline ticket to apply: 2 h 40 in business hours. Before HF-3105 it was 1.5 days because it waited for a backend ticket.

Of the 38: 29 "wrong load in the yard", 6 "pressed at pickup", 3 carrier admin confirmations on the wrong line. The mobile team has the split for the next UI review.

## Related

[[playbook-load-stuck-in-transit-after-delivery]] is the opposite direction. [[playbook-cancel-in-transit-two-person-rule]] is the other support-only state change and needs two people, this one does not, because it is reversible.

## Wording for the two parties

The macro `undeliver-done` says, in the ticket's language, roughly: the delivery recorded on <date> for load <reference> has been voided at your request (or at the carrier's request); the load is back in transit; the driver must confirm the delivery in the app when it takes place, and the delivery time will be the actual one; the POD photo previously attached remains in the record but is no longer shown as valid. It ends with the HF ticket key. For the shipper, one extra sentence when an invoice draft was deleted by billing: no invoice has been issued for this load.

Do not write "the error has been corrected" or attribute the mistake to the driver in writing to the shipper; the carrier reads the same thread in some cases (both parties on the load can be on the ticket), and the relationship between them is not ours to shape. Say what was done and what happens next.
