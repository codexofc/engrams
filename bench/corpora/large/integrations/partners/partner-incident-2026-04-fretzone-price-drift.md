---
name: partner-incident-2026-04-fretzone-price-drift
description: April 2026, Fretzone began rounding prices to the euro; our stale auto-fix looped, burned the daily quota in 3 hours and showed drifting prices
type: project
status: active
verified: 2026-05-08
---

# Incident 2026-04-21: the Fretzone price loop (HF-3135)

## Timeline (CEST)

- 08:10 Fretzone deploys a change on their side, not announced (it was not "incompatible" in their reading): listing prices are stored as integer euros, rounded half up.

- 08:15 Our pull (`app:partners:fretzone:pull`) fetches the status of our outbound listings and sees prices that differ from what we pushed (`1 234.50` pushed, `1 235` echoed). The pull's job for outbound listings is to detect partner-side edits and mark `PARTNER_STALE`; it did, for about 1 100 listings.

- 08:15 to 11:20 The reconciliation-on-stale logic (introduced in HF-3085 for Cargolink and reused for Fretzone) re-pushes our value for `PARTNER_STALE` rows. Fretzone accepts, rounds again, echoes `1 235`. Next pull, stale again, re-push. Every 5 minutes, 1 100 listings, each a push and a pull. Fretzone's daily quota of 20 000 calls ([[partner-fretzone-contract-quirks]]) is gone by 11:20.

- 11:20 Every Fretzone call returns 429. The pull fails, the push fails, `PUSH_FAILED` everywhere, alert `PartnerSyncErrors` fires.

- 11:35 On-call finds the 429s, thinks Fretzone is down, checks with their contact. Their contact says nothing is wrong on their side and mentions "we did clean up the price format this morning".

- 12:05 Root cause understood. Re-push on `PARTNER_STALE` disabled for Fretzone by flag `partners.fretzone.autofix_stale = false`. Quota still exhausted until midnight.

- 12:30 to 00:00 No sync with Fretzone. Shippers' price changes do not reach Fretzone; new Fretzone listings do not reach our carriers. Support macro sent to the 40 shippers with active Fretzone listings.

- 00:02 Quota resets, sync resumes, backlog of 9 hours processed in 25 minutes.

## What shippers saw

Between 08:15 and 11:20 the price on their load's Fretzone panel flickered between two values every few minutes, because we showed `PARTNER_STALE` with the partner's value. Six tickets, all "the Fretzone price keeps changing".

## Root causes

1. **Comparison without normalisation for Fretzone.** Cargolink's sync normalises before diffing (rounding to 5 EUR, their rule). Fretzone's rule was "cents accepted", so no rounding step existed. When their rule changed, our diff saw a change. A normalisation step is a copy of the partner's rules, and the partner can change them.

2. **Auto-fix on stale without a loop guard.** Re-pushing our value when the partner's differs is right when the partner lost an update. It is wrong when the partner *transforms* values. Nothing counted how many times the same row had been "fixed".

3. **No quota guard on our side.** We knew the quota and trusted our arithmetic (about 5 500 calls a day).

4. **Fretzone's change notice.** Rounding is a data change; their contract says 60 days for incompatible changes and they did not consider it one. We disagreed, politely, and they now announce any change to echoed values.

## Fixes

- Fretzone mapping rounds prices to the euro before push and before diff, matching their new rule ([[partner-load-field-mapping-rules]]).

- Stale auto-fix has a per-row counter: after 3 fixes in 24 hours for the same field, the row goes `PARTNER_STALE` with `last_error = 'stale_loop_suspected'`, no more pushes, alert to the team. It fired once since, in May, on a Cargolink listing where they had capped a price at their maximum; correct behaviour.

- Redis counter `partners:fretzone:calls:<date>`, hard stop at 18 000 with alert at 15 000. Same for Cargolink at their limit.

- Our shipper-facing panel shows our price and the partner's price separately with a note when they differ, instead of flickering.

## What we learned

- A partner's echo is not an edit. Any field a partner may normalise must be compared normalised, and the normalisation must be tested against their actual responses, not their documentation.

- An automatic corrective action needs a counter. Three times is a loop.

- "Nothing is wrong on our side" from a partner means "nothing we consider a change". Ask what they deployed.

## Figures

1 100 listings, about 14 000 wasted calls, 12 hours of degraded sync, 6 tickets, no load lost. The reconciliation ([[partner-reconciliation-nightly-job]]) the following night found 0 discrepancies, which was reassuring about the backlog handling.

## The detection gap, in detail

Three hours passed between the first phantom stale and the quota exhaustion, and nobody looked. The sync dashboard did show `PARTNER_STALE` jumping from about 20 to 1 100 at 08:15; there was no alert on that gauge because a stale row was considered benign (the auto-fix handles it). The auto-fix counter (`hf_partner_stale_fixes_total`) also climbed at 220 per 5 minutes, ten times its normal rate, with no alert either. The only alert that existed, `PartnerSyncErrors`, needs errors, and there were none until the 429s: every push succeeded, every pull succeeded, the loop was made entirely of successes.

New alerts since: `PartnerStaleSpike` when `PARTNER_STALE` rows exceed 200 or grow by 5 times in 15 minutes, and `PartnerAutoFixRate` when fixes exceed 50 per 5 minutes. Both would have fired at 08:20. Both are "notify", not "page": a partner-side change at 08:00 on a weekday is handled by the team at their desks, and if it happens on a Sunday the loop guard now stops it after three rounds without anyone.

## Communication with Fretzone afterwards

A written summary went to their technical contact and their account manager the next day, factual, with the request that any change to values they echo back be announced. Their answer added a "data format changes" section to their partner newsletter, which had not existed. We are on the list. It has been used twice since, once for a new optional field and once for a change in how they truncate long texts (from 300 to 280 characters), both of which the recorded-response suite would have caught within a week and now caught within a day of the newsletter.
