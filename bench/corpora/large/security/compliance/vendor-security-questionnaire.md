---
name: vendor-security-questionnaire
description: Customer security questionnaires are answered from a 140 question answer bank plus the TOM document; median 4 working days; two honest recurring gaps
type: reference
status: active
verified: 2026-06-12
---

# Answering customer security questionnaires

Enterprise shippers send a spreadsheet of 80 to 300 questions before signing. In 2025 we answered 17; in the first half of 2026, 14. The process below is what stopped it from eating a week each time.

## The answer bank

`halden-legal/questionnaires/answers.yaml`: 140 canonical questions with our answer, the evidence reference, the owner and the date last verified.

```yaml
- id: Q-IAM-04
  question: Do you enforce multi-factor authentication for administrative access?
  answer: >
    Yes. All staff access goes through our identity provider with mandatory
    MFA (TOTP or hardware key) since 2026-01-15; administrators and finance
    staff must use a hardware key.
  evidence: [TOM-2026-01 section 4.2, access-review-2026-Q1]
  owner: security
  verified: 2026-04-10
```

A new questionnaire is matched against the bank by the person on the compliance rota: most questions are rephrasings of a bank entry, and the answer is copied with the customer's numbering. Questions not in the bank get a new entry if they are likely to recur (rule of thumb: if the question is about us and not about a specific contract, it will recur). Free-text answers are always in English; two French customers received English answers and did not mind.

## The TOM document

`TOM-2026-01`, "Technical and organisational measures", 14 pages, is the attachment we send with every questionnaire. Sections: governance, access control, encryption (at rest by the storage layers, in transit TLS 1.2 minimum, certificate management), logging and monitoring, vulnerability management (see [[pentest-2025-10-findings]] and [[pentest-2026-04-findings]], summarised without the details), incident response, business continuity, sub-processors ([[dpa-telematics-subprocessors]] among them), data retention ([[data-retention-matrix]]), data subject rights. Revised twice a year, versioned by date, and the register ([[records-of-processing-register]]) references its id.

## Turnaround

Median 4 working days from receipt to sending, 2025 to mid 2026. The long tail (two at 15 days) were customers whose questionnaire was a portal with mandatory uploads of certificates we do not have.

## The recurring gaps

Honesty in the answer bank matters more than looking good. Two questions get a "no" or "partially":

- **MFA for customer users.** Not available in June 2026. The answer explains the SSO/SCIM path for enterprise customers (their own IdP enforces MFA) and states that native MFA is planned. Lost one deal over it in 2025, in the customer's own words.

- **A formal business continuity plan with tested RTO/RPO.** We have backups with tested restores and a documented failover, but not a BCP document in the shape auditors expect. The answer describes what exists with figures (last restore drill, measured RPO 5 min from WAL shipping) and says there is no formal BCP. Nobody has walked away over this one; three asked for the drill report and got it.

## Certifications

We have none and the bank says so. The question "Are you ISO 27001 certified" is answered "No. We follow the control set of ISO 27001 Annex A as a checklist; the mapping is in TOM section 11; we are not certified and have no date for certification". Some customers stop there. Those that continue typically ask for the pen-test summary and the access review evidence ([[access-review-quarterly]]), which we can give.

## Rules

- Never answer from memory. If the bank does not have it and no document supports it, write the document first or answer "no".

- Never send the full pen-test report. The executive summary and the remediation status table, yes.

- Every sent questionnaire is archived in `hf-compliance/questionnaires/<customer>/<date>/` with the version of the bank used, so that when the customer re-audits in a year we know what we said.
