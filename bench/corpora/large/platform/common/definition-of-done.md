---
name: definition-of-done
description: A ticket is done when merged to main, deployed to staging, checked there by the author, documented where needed, flagged if risky, and the ticket comment says how to verify in production
type: reference
status: active
verified: 2026-03-16
---

# Definition of done (platform)

A ticket moves to Done when all of this is true. Not before. "Merged" is not done.

1. **Merged to `main`** through a reviewed MR, see [[code-review-rules]].

2. **Deployed to staging** and **checked by the author** in staging, with the check described in a ticket comment: what was clicked or called, what was seen. A screenshot when it is UI. This step catches about one bug per week that the tests did not, usually configuration or data shape.

3. **Tests** at the right layer, see [[testing-pyramid-rules]]. The MR says which.

4. **Documentation** where it lives: the OpenAPI description for an endpoint, the glossary for a new term, the runbook for anything the on-call might touch, the project memory for a decision or a finding worth keeping. Not a wiki page nobody will find.

5. **Flag** if the change is risky (touches sync, invoicing, authentication, or the board's core interactions). The flag name is in the ticket and in the commit footer.

6. **Verification note for production**: one comment on the ticket saying how to verify it in production after promotion, and which dashboard or query shows it. The person promoting reads those.

7. **Cleanup ticket** created if the change leaves something temporary behind (a flag to remove, a dual-write to stop, a deprecated endpoint to sunset), linked with a `Depend` link.

## What done does not require

- Deployed to production. Promotion is by tag, batched, and decided by the release owner of the week. A ticket can be done and wait a few days.

- Product sign-off. Product looks at staging or the preview environment during development, not at the end.

## The comment that closes the ticket

Written in first person by the author, three lines: what I did, how I checked it in staging, how to check it in production. It is the trace that survives when the MR discussion is forgotten.

## Why this exists

The retrospective of September 2025 counted 11 tickets closed at merge that were found broken in production over the previous quarter, mostly configuration that existed in dev and not in staging. Step 2 removed almost all of them.
