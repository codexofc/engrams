---
name: billing-cutoff-timezone
description: The 1 January 2026 incident where the PL sequence reset at midnight UTC instead of Europe/Warsaw, 37 invoices numbered in the wrong year, and the rule that every billing cutoff is evaluated in the entity timezone
type: project
status: active
verified: 2026-01-09
---

## What happened

On 2025-12-31 at 23:00 UTC (00:00 Europe/Warsaw), the yearly reset of the invoice counter (see [[invoice-numbering-sequence]]) had not yet happened for the PL entity because `InvoiceSequenceAllocator` compared `now()` in UTC with `fiscal_year`. Between 23:00 and 00:00 UTC, 37 invoices issued by Polish shippers who close loads late in the evening (a lot of Polish carriers deliver overnight) got numbers `HF-PL-2025-00xxxx`, with an issue date of 2026-01-01 in Warsaw time on the PDF.

An invoice dated 2026 with a 2025 number is a sequence break in the eyes of the Polish tax administration (the JPK file is per month and the numbers must be monotonic with dates). The accountant asked us to cancel and re-issue all 37.

Same bug existed for FR and DE but did not trigger because nobody issues invoices at midnight on New Year in those entities. Pure luck.

## Fix (HF-2385)

- `InvoiceSequenceAllocator` now computes the fiscal year from `ZonedDateTime.now(entity.timeZone)`, with `legal_entities.time_zone` mandatory (`Europe/Paris`, `Europe/Berlin`, `Europe/Warsaw`, `Europe/Amsterdam`).
- The same rule applies to every other cutoff in billing: the day boundary for the debit batch, the month boundary for month close (see [[billing-runbook-month-close]]), the due date of an invoice (`issue_date + payment_terms_days` at 23:59:59 in the entity time zone), the dunning steps. A helper `EntityClock` wraps it and the old `Clock.systemUTC()` usages were removed from `billing/` in one sweep (48 call sites).
- A test `newYearInWarsawIsStillOldYearInUtc()` fixes the clock at 2025-12-31T23:30Z and asserts a 2026 number for PL and a 2025 number for a hypothetical entity in `Atlantic/Azores`.

## Re-issue

The 37 invoices were credited in full (credit notes numbered in 2026, reason `vat_error` for lack of a better enum value; we added `numbering_error` afterwards) and re-issued with 2026 numbers on 2026-01-02. Shippers were emailed once with both documents. Two of them had already paid; their payments matched the new invoices via the account balance mechanism of [[credit-notes-flow]].

## Why this note exists

The data warehouse stores everything in UTC and that is right for analytics, but billing is a legal process that happens in a place. Anyone touching a date in `billing/` should ask which entity's clock applies. The warehouse side of the same question is documented by the data team in their timezone convention note.
