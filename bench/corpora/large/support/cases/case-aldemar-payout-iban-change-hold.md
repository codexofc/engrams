---
name: case-aldemar-payout-iban-change-hold
description: March 2026, Aldemar Cargo changed its IBAN before a 31 000 EUR payout and hit the 48-hour hold; we kept the hold and fixed the screen
type: project
status: active
verified: 2026-05-14
---

# Case: Aldemar Cargo and the IBAN change hold

Fictional Spanish carrier, 45 trucks, Business plan, `payout:payment`. Ticket 2026-03-11, escalated to the account manager the same day.

## What happened

Aldemar's accountant changed the company IBAN in the settings on 10 March at 17:30, because they were switching banks. A payout batch of 31 240 EUR was scheduled for 11 March at 06:00. The hold introduced by HF-3095 after a fraud attempt in January (someone with a compromised admin login changed the IBAN of a carrier and the next payout went to them) blocks payouts for 48 hours after any IBAN change. The batch skipped Aldemar. The accountant saw "payout scheduled" turn into "payout on hold" with no explanation on the screen.

The owner called, then wrote, in that order. He needed the money to pay drivers on the 12th.

## What we did

- Support confirmed the hold, gave the release time (12 March 17:30, next batch 13 March 06:00), and explained the reason. Macro `payout-iban-hold` did not exist yet; it was written from this reply.

- The account manager offered nothing on the hold itself. Finance does not lift it, the backend cannot, that is the point. He arranged with finance a manual advance outside the platform on the 12th, which is not something we can promise in general and which we did not put in any macro.

- HF-3098: the settings screen now warns before saving an IBAN change: "Payouts will be held for 48 hours after this change. Next scheduled payout: 11 March 06:00 (31 240 EUR). Continue?" And the payout screen shows the hold reason and the release time.

- The e-mail confirming an IBAN change now goes to all admins of the org, not only the one who changed it. That was actually the anti-fraud value we had missed.

## What we learned

- A security hold without a visible reason looks like a bug. The hold was right; the screen was wrong.

- "We will not lift it, even for you" is the correct answer and it is easier to say when the screen said it before the customer clicked.

- The account manager's manual advance saved the account but it is not a procedure. We wrote it down here so nobody quotes it as one.

## Figures

Since the warning shipped (2026-03-25), IBAN changes fell from about 40 a month to 26; a share of them were mistakes or tests. Tickets about the hold: 6 in March, 1 in April, 0 in May and June. Aldemar renewed in June. See [[case-lessons-recurring-themes-2026-h1]] for where this sits among the year's lessons.
