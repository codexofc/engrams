---
name: compliance-team-preferences
description: The compliance rota wants every rule enforced by a job with a dry-run report, evidence as hashed files, tickets over spreadsheets, no ID scans for DSARs
type: user
status: active
verified: 2026-05-08
---

# How the compliance rota likes to work

Two engineers on rotation plus the external DPO. Not lawyers. These are the habits that make the work bearable.

- **A rule that is not enforced by a job is a wish.** Every retention line has a `RetentionRule` class and a nightly dry-run count (see [[data-retention-matrix]]). Every register entry is linted. If someone proposes a policy, the first question is "what runs it, and what reports when it does not".

- **Dry-run first, always, with the number in front of your eyes.** The homonym erasure of December 2025 ([[dsar-handling-runbook]]) is the reason `compliance:dsar:erase` refuses to run without a recent dry-run by the same person. We would rather be slow.

- **Evidence is a file with a hash in `hf-compliance/`.** Access review CSVs, register PDFs, DPA signatures, retention reports. A screenshot in a chat thread is not evidence six months later. The bucket is write-once by policy (object lock, 6 years).

- **Tickets, not spreadsheets.** Pen-test findings, access review lines, DSAR follow-ups: YouTrack, with due dates that do not move. The 2025 pen-test was the first one tracked this way and it is the first one we can actually report on.

- **We do not collect identity documents to verify a DSAR.** Proportionate verification (confirmation link, matching details) is enough for what we hold. Asking for an ID scan to delete a phone number is absurd and creates a new pile of sensitive data.

- **Legitimate interest with a written balancing test beats a consent nobody can refuse.** Drivers and their employer's tools are the standing example ([[dpo-feedback-position-data]]).

- **Generated over hand-written.** The data map comes from annotations, the cookie page from a YAML, the register PDF from the YAML register. Hand-written compliance documents drift within a quarter. When something cannot be generated (vendor consoles, the stores outside Postgres) we keep it in a small hand-maintained file next to the generated one and review both together.

- **Answer the security questionnaire from the documents, never from memory** ([[vendor-security-questionnaire]]). If a question cannot be answered from an existing document, write the document first, then answer.

- **English for register, DPAs and questionnaires, French for internal runbooks.** Customers' legal teams read English; our support reads French. Ticket language follows the platform's habit (French) except for pen-test findings, which are in English because the consultant's text is.

- **We say what we do not do.** Every note in this project has a section on what was refused and why. A future reader who wants to reopen a decision should find the argument, not just the outcome.

Pet peeve: being asked for "the GDPR compliance certificate". There is none. There is the register, the map, the matrix, the reviews, the tests and the DPAs, and we can show every one of them with a date.
