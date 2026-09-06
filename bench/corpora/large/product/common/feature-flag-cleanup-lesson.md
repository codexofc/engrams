---
name: feature-flag-cleanup-lesson
description: December 2025 flag audit: 61 dead flags of 143, two production bugs from stale branches, expiry and cleanup ticket now mandatory
type: feedback
status: active
verified: 2026-01-12
---

## What we found

The December 2025 audit of the `flags` table: 143 flags, 61 of them either at 100 % for more than 3 months or at 0 % with no evaluation in 30 days. 38 had no owner that still existed as a team. The oldest dead flag dated from 2023.

Two production bugs in 2025 came from stale flag branches:

- `dispatch.new_assignment_flow` at 100 % for a year; someone fixed a bug in the old branch (the one nobody ran) because the file still had both and the test exercised the old path by default.
- `billing.legacy_dunning` at 0 %, then flipped on by mistake from the flags page during an unrelated change, resurrecting a dunning ladder with a 7-day first reminder. 300 shippers received a firm reminder 3 days after their invoice. Credit notes were not needed but the apology email was.

## The rule

Now in [[feature-flags-convention]]: every flag has an expiry (90 days maximum, two extensions), an owner team, a kind, and the flags page creates the cleanup ticket at creation. Flags past expiry are listed in the weekly digest to the owning team. `ops` kill switches are the only permanent ones and they are reviewed once a quarter for whether they still guard a real failure mode.

## How to apply

- When you create a flag, write the removal ticket's description as if the flag were already at 100 %: which files, which tests, which branch to delete. Doing it later means rediscovering everything.
- When you fix a bug near a flag, check the flag's state in production first. If it is at 100 % or 0 %, delete the dead branch in the same change.
- Never flip a flag from the flags page without reading its description and owner. If the description is empty, that is the bug to fix first.
- Experiments end with a decision in the ticket and the flag removed within two weeks; the pricing team's experiment guidelines say the same from their side.

## Numbers after

March 2026: 52 flags, 0 past expiry, 9 `ops` kill switches. The digest goes out on Monday and takes the owning teams about 20 minutes a week in total.
