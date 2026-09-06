---
name: case-delacroix-rating-drop-shipper-cancels
description: May 2026, Delacroix Fret's rating fell 4.6 to 3.9 because a shipper's withdrawals were counted as carrier withdrawals; actor check added
type: project
status: active
verified: 2026-06-26
---

# Case: Delacroix Fret, a rating drop that was not theirs

Fictional French carrier, 35 trucks, Business, `other:rating`. Ticket 2026-05-19 from the owner, who noticed the drop because a regular shipper asked what had happened.

## What happened

A shipper (a food producer, not named) had a plant shutdown and needed to cancel six loads already `DISPATCHED` to Delacroix. Their dispatcher used the "Withdraw carrier" button on each load, then cancelled the now-`OPEN` loads. Six `withdraw_carrier` events, actor recorded as the shipper user, but the rating computation counted them as carrier withdrawals because, at the time, it did not look at the actor. Six withdrawals in a week on a 90-day window of about 110 loads: the rating went from 4.6 to 3.9.

Delacroix had done nothing. The shipper had not either, really: the UI offered two buttons and one of them looked less final.

## What we did

- L2 read `hfctl load events` on the six loads, saw the actor, and requalified each event with `hfctl rating requalify <event_id> --as shipper_cancel`. The rating recomputed overnight to 4.6.

- HF-3170 was already open about withdrawal penalties; this case added the actor check. Since 2026-06-08 `withdraw_carrier` initiated by a shipper user is written as `shipper_withdraw` and never counts against the carrier.

- The shipper's cancellation dialog now says, when a carrier is assigned, "Cancelling the load will notify the carrier and apply the cancellation fee" and the withdraw button has moved under a "More" menu with the text "Remove this carrier and reopen for bids". People picked "withdraw" because it sounded softer.

- No cancellation fee was applied to the shipper for the six loads, because the withdraw path did not trigger it. The account manager talked to the shipper; Delacroix received a goodwill payment from the shipper directly. Not our process, but worth knowing that the withdraw path was also a fee bypass, which HF-3170 closed.

## What we learned

- An event's actor is data; a computation that ignores it is a bug waiting for a case.

- A rating that can drop by 0.7 in a week from one customer's clicks is too sensitive to one relationship; the rating owners are looking at capping the contribution of a single shipper to 30 % of the window.

- The carrier learned about the drop from a customer. The rating screen now shows a change log with the loads that moved the number; the requalification is visible there too.

## Figures

After the requalification, 22 other carriers were found with shipper-initiated withdrawals counted against them over the previous 90 days, 61 events in total, all requalified in one batch by the backend on 2026-06-09. Two of them had opened tickets we had closed with the "the calculation is the same for everyone" macro. We wrote to both. This is the case that made the [[case-lessons-recurring-themes-2026-h1]] list say "read the actor before defending the calculation".
