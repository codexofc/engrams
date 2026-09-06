---
name: partner-onboarding-checklist-new-board
description: Checklist for a new load board: contract clauses into code first, mapping tests, refs table, quota guard, reconciliation, support, 6-week timeline
type: reference
status: active
verified: 2026-06-27
---

# Onboarding a new partner board: the checklist

Written after the second integration (Fretzone) so that the third does not repeat the first two's mistakes. Sales keeps asking ([[partner-feedback-sales-wants-more-boards]]); this is what "yes" costs.

## Before any code

- **Read the contract with the integrations team present.** Extract every clause that has a runtime effect into a note like [[partner-fretzone-contract-quirks]] before the mapping is designed: listing lifetime, exclusivity windows, price rules (floors, rounding, displayed margins), call quotas, data retention, no-solicitation, commission rules and exemptions, change notice period. Cargolink's 14-day lifetime was in the contract and not in the code for two months; that was the December incident.

- **Get their sandbox and a technical contact** before signing, and send one listing through by hand. Their documentation will differ from their API in at least three places; find them early ([[partner-sandbox-environments]]).

- **Decide the direction of each flow** (loads out, loads in, bids, acceptances, documents, tracking) and write it in the overview note. "Both directions" for loads means dedup work; budget it.

## Code

- `PartnerListingMapper` implementation with a contract test per event and per field family (vehicle types, ADR, addresses, prices, time windows, text), including the partner's **normalisation** so that our diff compares like with like ([[partner-load-field-mapping-rules]]).

- Rows in `partner_load_refs` from day one, with `previous_external_ids` handling and a resolver that never answers 404 without checking history ([[partner-dedup-external-refs-table]]).

- Event-driven push from the start (a `PartnerSyncRequested` handler), not a periodic comparison. The comparison is the nightly reconciliation, not the primary path.

- Per-load advisory lock in the handler.

- Stale auto-fix with the 3-per-24h loop guard.

- Quota guard in Redis at 90 % of their daily or per-minute limit, with an alert at 75 %.

- Inbound: shadow carrier provisioning with registration-number matching, `no_solicitation_until` if the contract says so, purge job at the contractual retention.

- Reconciliation: extend `app:partners:reconcile` with the partner's listing endpoint and commission rules, and the statement generator.

- Metrics with the `partner` label on the existing series; alerts inherit.

- Feature flag `partners.<name>.enabled` per organisation, so the rollout is a handful of friendly shippers first.

## Support and customers

- Update the support playbook for partner mismatches with the partner's error codes in words, and the macro list.

- Shipper-facing panel: partner status per load with `last_error` in words from day one. Do not make support discover errors in the table.

- Help centre article for carriers if loads flow in (exclusivity delays, displayed margins), and for shippers (which fields are not sent, why a load was not pushed).

- Reconciliation statement agreed with the partner's finance before the first month closes: who counts what, in which timezone (UTC, we learned).

## Timeline that actually happened

Cargolink: contract to first production listing 7 weeks, to stable 4 months (the v1 sync rewrite). Fretzone: 5 weeks to production, 3 months to stable (dedup rule). A third board with this checklist: estimate 6 weeks to production, stable at 8 weeks if the contract has no surprises. Sales has been told 3 months to be safe.

## When to say no

A board without a sandbox, without a machine-readable listing status, or without a stable listing id we can store, is a manual integration dressed up as an API. Two candidates in 2026 were declined on that basis; both were told what would change our mind.
