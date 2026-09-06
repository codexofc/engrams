---
name: code-review-rules
description: One approval from a non-author, review within one working day, comments tagged blocking or nit, author merges, reviewers read the tests first, and the four things that always get a comment
type: reference
status: active
verified: 2026-02-11
---

# Code review rules (platform)

## Mechanics

- One approval from someone who did not write the code. Two for anything touching authentication, invoicing, migrations on the big tables, or the sync protocol of the driver app. The MR template has a checkbox that sets the required count.

- A review request is answered within one working day. Not "reviewed", answered: a "will look tomorrow afternoon" is fine. The reviewer of the week (rotation in the team calendar) is the default when the author does not know who to ask.

- The author merges, after approval and green CI. Not the reviewer. The author knows if something else is pending.

- Comments carry a prefix: `blocking:` (must change before merge), `question:` (needs an answer, not necessarily a change), `nit:` (author decides), `praise:` (yes, really, we want those). An unprefixed comment is a `question:`.

- A `blocking:` comment is resolved by the reviewer who wrote it, not by the author. Others by the author.

- Disagreement that survives two rounds goes to a 15 minute call, and the outcome is written in the MR.

## What reviewers do first

Read the tests. If the tests describe the behaviour clearly, the implementation is read quickly. If there are no tests, the first comment is `blocking: where are the tests` unless the MR says why (a pure refactor covered by existing tests, a config change).

Then run it, when it is a UI change. The debug APK or the preview environment link is in the MR for that purpose.

## Four things that always get a comment

1. A migration without the estimation line (rows in prod, expected duration). See the API migration workflow.

2. A new dependency without a line saying why the existing ones do not do it, and what it weighs.

3. A `TODO` without a ticket key. `TODO(HF-1234):` is fine, `TODO: fix later` is not.

4. A catch block that swallows an exception without a comment explaining what is being ignored and why.

## What reviewers do not do

- Restyle code that the formatter accepted. If it bothers you, change the formatter config in a separate MR.

- Ask for a different architecture on a 30 line fix. Open a ticket.

- Approve with `nit:` comments unresolved and then complain later.

## Size

Above 400 lines changed (excluding generated files and lock files), the reviewer may ask for a split, and usually does. The Symfony 7 upgrade MR was 340 files and got an exception with two reviewers reading different halves.

Related: [[branching-and-pr-flow]], [[definition-of-done]].
