---
name: first-load-nudge-experiment
description: HF-2575: showing open loads on declared lanes before the licence lifted uploads at day 3 from 66 % to 73 %, shipped June 2026
type: project
status: active
verified: 2026-07-06
---

## Hypothesis

From [[support-findings-onboarding-2026-04]]: a fifth of carriers who stall before uploading the licence wanted to see whether there was work for them first. Showing real open loads on their lanes, before asking for the licence, should turn "I will see later" into either a decision to finish or an informed exit.

## Setup

- At sign-up the carrier already declares up to five lanes (origin region, destination region) and a vehicle type. Variant B shows, on the home screen right after step 2, the five most recent open loads matching those lanes, with the suggested price, and a locked "Bid" button that says what is missing ("Send your licence to bid").
- Variant A: the existing home screen (a checklist of steps).
- Randomised by `carrier_id`, 50/50, 2026-05-04 to 2026-06-14, all countries. 3 620 sign-ups in the period.
- Primary metric: licence uploaded within 3 days of step 2. Secondary: first bid within 14 days, first award within 21 days, and the share of accounts that never come back after seeing the loads (the informed exit).

## Results

| Metric | A | B |
|---|---|---|
| licence uploaded at day 3 | 66.0 % | 73.1 % |
| first bid at day 14 | 47.6 % | 51.9 % |
| first award at day 21 | 30.9 % | 33.4 % |
| no return after step 2 | 14.2 % | 15.8 % |

The informed exit went up a little (carriers who saw few loads on their lanes left sooner), which we consider a good outcome: they would have left anyway at day 21 after seven emails.

By country, the effect is largest in RO and LT (lanes towards western Europe with plenty of loads) and nearly zero in DE (the licence step is manual there anyway, see [[licence-community-check]], so speeding up the upload does not speed up the pass).

## Detail that matters

Carriers whose declared lanes matched fewer than 3 open loads saw a different message in B: "Few loads on your lanes right now, here are nearby lanes with open loads" with the nearest alternatives. Without that fallback (first week of the experiment, before HF-2581), those carriers converted worse in B than in A (58 % versus 64 % on licence upload), because an empty screen is a stronger signal of "nothing for you" than a checklist. The fallback is now permanent and applies to the day 4 email of [[onboarding-email-sequence]] too.

## Decision

Shipped to 100 % on 2026-06-22. Flag removed in HF-2598. The Q2 funnel targets ([[activation-funnel-q1-2026]]) were 52 % first bid and 34 % first award; the experiment alone brings 51.9 % and 33.4 %, so the Q2 measurement will tell whether the copy change at step 2 adds the rest.
