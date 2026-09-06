---
name: playbook-shipper-cannot-cancel-load
description: Shipper cannot cancel: allowed source states, fee once DISPATCHED, IN_TRANSIT is support-only, and the wording when the answer is no
type: reference
status: active
verified: 2026-05-28
---

# Shipper cannot cancel a load

Category `load:cancel`. The shipper clicks "Cancel" and gets a refusal, or the button is missing. What they can do depends on the load's state, and the state machine does not bend.

## By state

`hfctl load get <load_id>` first.

- `DRAFT`, `OPEN`, `BIDDING`: cancel is allowed and free. If the button is missing, it is a permission problem: the user needs the `shipper_admin` or `dispatcher` role in their org (`hfctl org users`). Macro `cancel-role-needed`, their admin grants the role.

- `DISPATCHED`: cancel is allowed, with the cancellation fee from the accepted bid's terms (`cancellation_fee_pct`, default 15 %, shown in the confirmation dialog). Refusals here are usually the user closing the dialog when they see the fee, then telling us the button "does not work". Macro `cancel-fee-explain`. The fee is between shipper and carrier; we invoice it on the carrier's behalf on the next run.

- `IN_TRANSIT`: the shipper cannot cancel. Goods are on a truck. Only support can, with the two-person rule, and only for the reasons listed in [[playbook-cancel-in-transit-two-person-rule]]. "The customer changed their mind" is not one of them. Macro `cancel-in-transit-refused`, then the shipper talks to the carrier about a return or a re-delivery, which is a new load.

- `DELIVERED`, `INVOICED`: nothing to cancel. Disputes go to [[playbook-invoice-dispute-amount]].

- `CANCELLED`: already done. Sometimes by the API integration without the user knowing; `hfctl load events` shows the actor.

## Errors from the API

- `transition_not_allowed`: state does not permit it. See above.

- `cancel_reason_required`: the API call had no `reason`. Integrator problem, macro `api-cancel-reason`.

- `load_locked`: a migration or a support action holds the load for a few seconds. Retry. If it persists over a minute, escalate.

## Bulk cancellations

A shipper who wants to cancel 40 loads because of a strike or a plant closure does it from the list view with multi-select (Business plan and above). We do not run bulk cancellations for them; the Meridian Agro case in the cases collection is why (they asked us to cancel "all loads for next week" and meant one site).

## Saying no

The refusal for `IN_TRANSIT` generates the angriest tickets we get. The macro is deliberately short and factual: state, who can act, what the alternative is. Do not argue the state machine, do not promise to "check with the team" when the answer is no. An Enterprise shipper who escalates goes to the account manager.
