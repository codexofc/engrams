---
name: postmortem-template-rules
description: A post-mortem has 7 fixed sections, a UTC timeline with a source per line, a root cause phrased as a system property and actions as dated tickets
type: reference
status: active
verified: 2026-03-20
---

# Post-mortem template and rules

The template is `halden-security/postmortems/TEMPLATE.md`. Post-mortems live in that repository as `YYYY-MM-DD-<slug>.md`, one MR each, reviewed by the incident commander and one person who was not involved. The rules below are the ones people ask about.

## Sections, in this order

1. **Summary**: five lines maximum. What happened, impact, duration, severity, whether personal data was involved. Written last, read first.

2. **Impact**: what was exposed or lost, to whom, for how long. Numbers with their uncertainty ("about 2 300 drivers, exact count in the query attached"). If the answer is "we do not know", say so and say why.

3. **Timeline**: see below.

4. **Root cause**: phrased as a property of the system, not as a person's action. "The bucket policy allowed public listing and nothing checked bucket policies" rather than "X made the bucket public". If a human action is part of the chain, the question the section answers is "what made that action possible and undetected".

5. **What went well**: at least one line. Detection that worked, a runbook that helped, a fast rollback. Not for morale, for knowing what to keep.

6. **What went badly**: detection gaps, slow steps, missing tools, wrong assumptions.

7. **Actions**: table with ticket key, description, owner, due date, status. No ticket, no action. Actions are split into "prevents recurrence", "improves detection", "improves response". A post-mortem with only the first kind is sent back: the incident will happen again in a different shape and the second and third kinds are what will save us.

## Timeline rules

- **UTC**, always, even though the team works in three time zones. Local times in parentheses only when they matter (a click at 08:02 local explains why it was the first email of the day).

- One line per event with a **source**: `[chat]`, `[audit]`, `[log]`, `[ticket]`, `[memory]`. A `[memory]` line is allowed but marked, because memories of an incident are unreliable within hours.

- Include the boring lines: when the page fired, when it was acknowledged, when the commander was named, when containment was declared, when the incident was closed. The gaps between those are the response metrics.

- The scribe's live log ([[incident-timeline-tooling]]) is the primary source; the post-mortem timeline is a cleaned copy, not a rewrite.

## Language

English for the post-mortem body, because the security questionnaire process may share a summary with customers. Quotes from the chat stay in their original language.

## What is not in a post-mortem

- Names of people. Roles instead: "the on-call engineer", "a finance team member", "the commander".

- Speculation about intent of an external attacker beyond what the evidence shows.

- Secrets, even revoked ones. Prefix and last four characters at most.

- Vendor blame without our part: if a vendor failed, the section says what we assumed about the vendor and what we will check from now on.

## Review

The reviewer who was not involved checks one thing above all: could a reader in a year understand what happened without asking anyone. If not, the post-mortem is not done. The SEV1 review meeting ([[incident-process-severity-levels]]) discusses the actions, not the narrative; the narrative should be finished before the meeting.

## Examples worth reading first

- [[incident-2025-12-test-credentials-build-log]]: a clean example of a root cause phrased as a system property.

- [[incident-2026-02-pod-bucket-public]]: the longest one, with the notification track running alongside.
