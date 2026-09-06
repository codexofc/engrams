---
name: case-feedback-carriers-want-phone-support
description: Small carriers keep asking for phone support (survey and 60 tickets); why we kept it to Enterprise and the 2-hour callback we built instead
type: feedback
status: active
verified: 2026-06-30
---

# Feedback: small carriers want to call us

Not one case, a pattern. In the 2026 survey, 34 % of carrier respondents on Starter and Business asked for phone support. In Deskline, about 60 tickets in H1 2026 contain a variant of "can I just call someone", almost all from carriers with fewer than 20 trucks, almost all about a driver blocked right now.

## Why they ask

A dispatcher at a small carrier has a driver at a loading dock who cannot press pickup, the shipper's forklift operator waiting, and a ticket form asking for a `load_id`. Writing is slow, they are on the phone with the driver already, and they want to hand the driver over to someone who can fix it. It is a fair need, and our SLA (8 business hours for Starter) is not designed for it.

Some quotes, translated:

- "I do not have time to type when the truck is at the dock."

- "Your form asks me for an ID I have to find on the computer I am not in front of."

- "With the other load board I call and someone picks up."

## Why we said no to a phone line

Discussed twice in the [[case-lessons-recurring-themes-2026-h1]] period, decided in the triage of 2026-03-16 with the support lead and product:

- Six agents, three languages, five countries. A line with a decent pickup rate needs at least two people dedicated during the day. That is a third of the team not writing.

- Most "blocked at the dock" cases are solved by the carrier themselves once they know the two moves (send the PIN link, ask the driver to open the app on network). Teaching is a better investment than picking up.

- Phone leaves no record, and our fixes need identifiers anyway; the call would end with "please send me the load id".

## What we did instead

- **Callback within 2 hours** for tickets tagged `driver:login` or `driver:sync` with the flag "driver is at the dock now", on all plans, during business hours. The form has a checkbox; L1 calls back. Measured since April: median callback 24 minutes, 310 callbacks in Q2, 78 % solved during the call.

- The ticket form on mobile (carriers write from their phone) accepts a photo of the driver app's screen and reads the `load_id` from the QR code shown on the load screen since 4.8. No more "find the id on the computer".

- The carrier back-office got the two self-service moves as buttons with a one-line explanation (send PIN link, HF-3155; view driver sync status, HF-3157). Tickets `driver:login` from carriers who used the buttons fell by half.

- The macro for these tickets starts with the sentence "If the driver is at the dock now, tick the callback box and we call you within two hours."

## What we watch

Whether the callback volume grows past what two agents can absorb in a morning. At 310 per quarter it is fine. At 1 000 the phone line question comes back, and this note will say so.
