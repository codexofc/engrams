---
name: security-champions-program
description: One security champion per team (7) reviews security-tagged MRs, runs lite threat models and gets 10 % time; 14 issues caught before merge in H1 2026
type: project
status: active
verified: 2026-06-30
---

# Security champions (HF-2120)

There is no security team. There is a rota ([[security-oncall-rota]]) and there are champions: one engineer per team who spends part of their time on the security of their team's code. Started February 2026 after the pen-test and the incidents of late 2025 made it clear that security review could not be a bottleneck on two people.

## Who

Seven champions in June 2026: API, web front, mobile, data platform, ops, telematics integrations, EDI integrations. Volunteers, confirmed by their team lead, for a year. Not the most senior person by default; the mobile champion was 18 months in and is the one who found the certificate-pinning kill switch gap.

## What a champion does

- **Reviews every MR tagged `security`** in their repository. The tag is set by the author when the change touches authentication, authorization, secrets, file handling, external input parsing, or a dependency change outside the weekly bot MR. Missing tag found in review: the reviewer adds it, and the champion reviews anyway. Median added review delay: 6 hours.

- **Runs the lite threat model** ([[threat-model-lite-template]]) with the feature owner for anything new that handles personal data, money, or external input. One hour, one page.

- **Owns the checklist** for their stack: the PHP one is [[secure-coding-checklist-php]], the front and mobile ones live next to it.

- **Is the first call** from the rota when an incident touches their area.

- **Spends 10 % of their time** on it, agreed with the team lead and visible in planning. When the 10 % is regularly exceeded (it was for the API champion in March), the work becomes a ticket for the team, not overtime for the champion.

## Monthly session

One hour, first Thursday, all champions plus the rota of the week. Agenda: one incident or pen-test finding read together, one topic (in H1 2026: SCIM filter parsing, presigned URLs, HMAC verification pitfalls, the registry proxy, breach notification from the engineer's side, PKCE), and the open questions from the month. Notes in `halden-security/champions/2026-06.md`.

## Results, H1 2026

- 14 issues caught in review before merge, by champions, that a non-champion reviewer had approved or would likely have approved. Examples: a `runUnscoped` call without justification in a new report endpoint; an HMAC compare with `==`; an upload endpoint accepting `.svg` for avatars (scripts); a Messenger handler logging a full request body including a password reset token.

- 9 lite threat models run, 4 of which changed the design (one feature moved from storing a document to storing a reference to it; one dropped a "share by email with anyone" option).

- Zero champion-reviewed MRs among the causes of the H1 incidents. Correlation, not proof, and the sample is small.

## What did not work

- A shared "security backlog" board for champions. Nobody looked at it; work happens in the team's own board with a `security` tag.

- Asking champions to write the monthly SBOM review. Moved to the rota; the champions' time is better spent on their own code.

- A certification-style training. Two champions did an external course; both said the monthly session with a real incident taught more.

## Rules

- A champion can be overruled by their team lead on priority, never on the content of a security review. Disagreements go to the monthly session, and if unresolved, to the head of engineering with both positions written.

- Champion rotates after a year unless they want to continue; the point is spreading knowledge, not creating seven part-time security engineers.

- The security rota is staffed from the champions plus the ops volunteers, so every champion does a few rota weeks a year and sees the alerts their code produces.
