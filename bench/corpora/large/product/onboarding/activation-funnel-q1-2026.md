---
name: activation-funnel-q1-2026
description: Q1 2026 carrier funnel: 5 480 sign-ups, 47.6 % first bid, 31.4 % first award, biggest drop at licence upload, country split
type: project
status: active
verified: 2026-04-14
---

Figures for carriers who signed up between 2026-01-01 and 2026-03-31, measured on 2026-04-10 (so the March cohort had at least 10 days). Replaces the 2025 figures in [[onboarding-funnel-old-figures]].

## Funnel

| Stage | Count | % of sign-ups | % of previous |
|---|---|---|---|
| sign-up (email verified) | 5 480 | 100 | |
| identity passed (step 1) | 4 710 | 85.9 | 85.9 |
| company passed (step 2) | 4 020 | 73.4 | 85.4 |
| licence uploaded | 3 190 | 58.2 | 79.4 |
| licence passed (step 3, can bid) | 2 980 | 54.4 | 93.4 |
| first bid | 2 610 | 47.6 | 87.6 |
| first awarded load | 1 720 | 31.4 | 65.9 |
| insurance passed (step 4) | 2 150 | 39.2 | |
| bank passed (step 5) | 1 980 | 36.1 | |

The stages are those of [[carrier-verification-steps]]. The largest single loss is between company passed and licence uploaded: 830 carriers (20.6 % of those who passed step 2) never upload a licence. Support findings on why are in [[support-findings-onboarding-2026-04]].

## By country

| Country | Sign-ups | First bid | First award |
|---|---|---|---|
| PL | 2 140 | 51.2 % | 35.0 % |
| FR | 1 380 | 46.8 % | 30.1 % |
| DE | 810 | 40.1 % | 26.4 % |
| RO | 520 | 49.0 % | 33.5 % |
| LT | 310 | 52.9 % | 37.4 % |
| other | 320 | 38.4 % | 22.8 % |

German carriers convert worst. The licence step passes automatically less often in DE (44 % against 71 % in PL) because the German register is not queryable online and every licence goes through the manual queue.

## Time

- Median sign-up to first bid: 3.1 days (Q4 2025: 4.4 days). The improvement is the HEIC conversion and the 3-attempt call rule of [[document-check-rules]], which both shipped in January.
- Median first bid to first award: 2 days, 6.2 bids. New carriers win 18 % of their bids against 24 % for established ones; the reliability prior in the bid ranking is part of it.

## What changed compared with 2025

- Licence passed rate among uploads went from 87 % to 93.4 % (better pre-checks, fewer rejections for company name mismatch).
- First-award rate went from 27.9 % to 31.4 %.
- Sign-ups grew 41 %, mostly PL and RO, from the referral programme launched in November.

## Targets for Q2

First bid at 52 %, first award at 34 %. The two levers: the licence upload drop (an experiment on uploading the licence during sign-up instead of after, HF-2570) and the first-load nudge ([[first-load-nudge-experiment]]).

## Method

Cohort: carrier accounts whose `email_verified_at` falls in the quarter, excluding internal test accounts (dictionary `internal_accounts` in the warehouse) and accounts flagged fraud by the end of the measurement. Stages are the first time the event occurred, from `core.carriers` versions and the product events `verification.passed`, `bid.placed`, `load.awarded`, measured on 2026-04-10.

Query in `analyses/onboarding/funnel-2026-q1.sql`; it runs in 6 s on the warehouse and is reused with a date parameter every quarter. The stages are reported in absolute counts and in percent of sign-ups; the "% of previous" column is what to look at to find where to work.

## Segments that differ from the total

- **Referral sign-ups** (28 % of the quarter, from the November programme): first bid 56 %, first award 38 %, both 8 points above the rest. They also upload the licence faster (median 1.1 days against 2.6). The referrer usually tells them what to prepare.
- **Sole traders** (37 % of sign-ups, mostly PL and RO): first bid 49 %, in line, but insurance passed only 31 % against 45 % for companies; their certificates are more often the wrong kind (liability only, no CMR cover).
- **Sign-ups from the mobile app** (41 %): first bid 44 % against 50 % on web. The mobile document upload was the HEIC problem until January; the March mobile cohort alone is at 48 %.
- **Fleets declaring more than 20 trucks** (4 % of sign-ups): first bid 61 %, first award 49 %, and 70 % of them go through the manual queue for the licence because their documents are scans of an original held at the head office, which the copy rule now covers.

## Time to value

Median days from sign-up: identity 0.1, company 0.4, licence uploaded 1.9, licence passed 2.3, first bid 3.1, first award 5.2, insurance passed 4.8, bank passed 6.0. The bank step is last because carriers give the IBAN when they realise a payout is pending, which is the day-7 email's job ([[onboarding-email-sequence]]).

## Caveats on the numbers

- The March cohort had only 10 days at measurement; its first-award figure (28 %) will rise, and the quarter's 31.4 % is a floor. Re-measured on 2026-05-15: 32.6 %.
- `first bid` counts any bid, including one that was immediately withdrawn; 3 % of first bids are test bids that the carrier withdraws within a minute.
- Country is the company's registration country, not the lanes; a Lithuanian carrier working France to Spain is `LT`.
