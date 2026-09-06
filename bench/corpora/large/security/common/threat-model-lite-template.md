---
name: threat-model-lite-template
description: A one-page threat model in 5 questions (data, entry points, attacker goal, controls, detection) done in 1 h before building anything touching data or money
type: reference
status: active
verified: 2026-05-15
---

# Lite threat model

Full threat modelling with diagrams and STRIDE tables did not survive contact with a team shipping weekly. The lite version is one page, five questions, one hour, and it has been done 9 times in H1 2026 with the champions ([[security-champions-program]]). Template at `halden-security/threat-models/TEMPLATE.md`, filled copies next to it named after the ticket.

## When

Before building, for a feature that:

- stores or displays personal data not already handled the same way elsewhere;

- moves money or changes who gets paid;

- accepts input from outside our systems (file upload, webhook, EDI message, telematics feed, public form);

- adds a new way to authenticate or authorise anything;

- adds a third-party service.

Not for: a new column on an existing entity, a UI change, a refactor. The champion decides in doubt.

## The five questions

**1. What data does this touch, and whose?** List tables, stores and external calls. Use the compliance project's data map categories (identity, contact, location, documents, financial...). If the answer includes a category new to the codebase, the compliance rota gets a heads-up now, not at release.

**2. Who can reach it, and through what?** Every entry point: endpoint, message handler, CLI command, scheduled job, file drop. For each: which principal type (user, staff, API key, service account, integrator, anonymous) and which permission. "Anonymous" and "any authenticated user" are the answers that get the most attention.

**3. What would someone want, and who?** Three lines maximum. A competitor wanting prices; a fraudster wanting a payout redirected; a disgruntled ex-employee wanting to embarrass; a bored person with a scanner. Naming the motivation makes the next question concrete.

**4. What stops them?** For each entry point from question 2, the control: permission check, tenant filter, signature verification, rate limit, input validation, encryption. Then the honest part: where is the control implemented, and is it the **only** implementation of that behaviour (the incidents project has two incidents from duplicated security code paths). A control that exists "in the UI" is not a control.

**5. What tells us if it fails?** The audit event, the alert, the report line. If the answer is "nothing", that is a finding, and the ticket gets a sub-task. Most of the 9 models produced at least one detection sub-task; that is the question's job.

## Output

The filled page, committed, linked from the feature ticket. Plus the sub-tasks it produced. Plus one line in the MR description: "threat model: <link>". The reviewer checks the line exists for MRs that should have one.

## What changed because of it (examples)

- A "share this load with a partner by email" feature was going to email a link valid 30 days to any address. Question 3 ("who would want this": anyone forwarding the email) and question 4 (nothing after the first click) turned it into a link that requires the recipient to have an account, valid 7 days, revocable, audited.

- The EDI inbound gateway's file drop (question 2: anonymous, via SFTP with per-partner keys) got a size limit, a schema pre-check before parsing, and an alert on parse-failure rate, all from question 5.

- A driver document upload feature was going to accept any image format. Question 4 named the SVG-with-script case; the accepted list is JPEG, PNG, PDF, re-encoded server-side.

## Rules

- One hour. If it takes longer, the feature is too big for one model, split it.

- The feature owner writes, the champion asks. Not the reverse.

- No scoring, no likelihood-times-impact matrices. Either something is missing or it is not.

- Revisit when the feature changes in scope, not on a calendar.

The checklist ([[secure-coding-checklist-php]]) is what the reviewer uses at MR time; the threat model is what happens before the code exists.
