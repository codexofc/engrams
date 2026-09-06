---
name: support-macros-tone-feedback
description: Carrier feedback (April 2026 survey, 212 answers) that macro replies feel robotic, leading to shorter macros, a mandatory personal first line and the removal of the "we understand your frustration" opener
type: feedback
status: active
verified: 2026-05-05
---

# Feedback: our macros sound like a machine

In April 2026 we sent the yearly support survey to carriers and shippers who had opened at least one ticket in the previous six months. 212 answers, 61 % carriers. Overall satisfaction 3.9 / 5, fine. The free-text field was less fine.

## What they said

Recurring phrases, translated where needed:

- "I got the same paragraph three times, from two different people."

- "The answer starts by telling me you understand my frustration. You do not. Just tell me where my payout is."

- "I wrote in German, got an answer in English that quoted a French error message."

- "Your reply was correct but I had to read it twice to find the part that concerned me."

- One dispatcher at a mid-size Polish carrier (anonymised in the cases as Kowalczyk Logistik) wrote a full page. Summary: the fix was right, the tone made them feel like a ticket number.

Nobody complained about the fixes themselves. Resolution quality was rated 4.3 / 5. Tone was 3.2 / 5.

## What we changed

Decided in the [[support-weekly-triage-ritual]] of 2026-04-27, applied in the `support-macros` repo over two weeks:

- Every macro starts with a `{{first_line}}` placeholder that Deskline refuses to send empty. The agent writes one sentence in their own words about this specific case. It takes 20 seconds and it is the sentence people read.

- Removed "we understand your frustration", "thank you for your patience" and "we apologise for any inconvenience" from all 84 macros. An apology, when due, is written by hand and says what for.

- Macros were cut to the structure from [[support-team-preferences]]: what we saw, what we did, what you should do, when we come back. Median macro length went from 11 lines to 6.

- Error messages quoted in a reply are translated or explained, never pasted in another language than the ticket's.

- German macros were rewritten by the two German-speaking agents instead of being translated from French. Same for English, which had French sentence structure all over it.

## Did it work

Too early for the yearly survey. The post-ticket rating (one question, thumbs) went from 78 % positive in March to 86 % in June on the same volume. Reopen rate did not move, which is expected: the fixes were already right.

## What we did not do

We did not remove macros. A macro is what keeps the checks in the right order and the wording correct on legal points (VAT, cancellations). The problem was never the macro, it was sending it naked.
