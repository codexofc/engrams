---
name: load-status-state-machine
description: Load lifecycle (DRAFT, OPEN, BIDDING, DISPATCHED, IN_TRANSIT, DELIVERED, INVOICED, CANCELLED) enforced by Symfony Workflow in the entity, with the list of allowed transitions and who may trigger them
type: reference
status: active
verified: 2026-04-21
---

# Load state machine

Defined as a Symfony Workflow (`state_machine` type) named `load` in `config/packages/workflow.yaml`, marking store on `Load::$status` (backed enum `LoadStatus`). The entity method `Load::apply(string $transition)` is the only way to change status. Direct `setStatus()` does not exist.

## States

| State | Meaning |
|---|---|
| `DRAFT` | created by the shipper, not visible to carriers |
| `OPEN` | published, no bid yet |
| `BIDDING` | at least one bid |
| `DISPATCHED` | a bid was accepted, carrier assigned |
| `IN_TRANSIT` | driver confirmed pickup in the app |
| `DELIVERED` | driver confirmed delivery, POD may still be missing |
| `INVOICED` | invoice finalized |
| `CANCELLED` | terminal, any reason |

## Transitions

| Transition | From | To | Who |
|---|---|---|---|
| `publish` | DRAFT | OPEN | shipper |
| `first_bid` | OPEN | BIDDING | system (on bid creation) |
| `accept_bid` | BIDDING | DISPATCHED | shipper, or system for auto-accept |
| `withdraw_carrier` | DISPATCHED | OPEN or BIDDING (depends on remaining bids) | carrier admin, penalized in the rating |

## Transitions (driver, system and support)

| Transition | From | To | Who |
|---|---|---|---|
| `pickup` | DISPATCHED | IN_TRANSIT | driver |
| `deliver` | IN_TRANSIT | DELIVERED | driver |
| `invoice` | DELIVERED | INVOICED | system (invoice finalization) |
| `cancel` | DRAFT, OPEN, BIDDING, DISPATCHED | CANCELLED | shipper, or support |
| `cancel_in_transit` | IN_TRANSIT | CANCELLED | support only |

`withdraw_carrier` is the odd one because the target depends on data. It is implemented as two workflow transitions (`withdraw_to_open`, `withdraw_to_bidding`) and `Load::withdrawCarrier()` picks one. The rating impact is described in [[carrier-rating-computation]].

There is no transition out of `DELIVERED` except `invoice`. A delivery reported by mistake is corrected by support through a direct `load_events` edit and an `UPDATE`, logged in the audit log, and it happens roughly twice a month. A proper `undeliver` transition is HF-1590, not scheduled.

## Guards

Symfony Workflow guards (`workflow.load.guard.pickup` etc.) check the actor's role from the security token. Business preconditions (a bid must exist, the carrier must have valid insurance documents) are checked in the entity method before `apply()`, and throw a `DomainException` subclass that maps to 409 in the error envelope.

The guard does not protect against concurrency. See [[bid-acceptance-race-condition]] for why a database constraint is needed on top.

## Events

Every completed transition writes a `load_events` row (kind = transition name, actor, occurred_at, payload jsonb) through `LoadTransitionSubscriber` on `workflow.load.completed`. This table is the source for the carrier rating, the timeline in the web UI and the webhook events. It is partitioned, see [[audit-log-table-partitioning]].

## Rendering

`bin/console workflow:dump load | dot -Tsvg > docs/load-workflow.svg` is in the `docs` make target, and the SVG is in the repo. When the YAML changes, regenerate it, the diff shows in review.
