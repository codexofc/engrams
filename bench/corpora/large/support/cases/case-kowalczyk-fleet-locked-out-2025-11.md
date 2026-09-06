---
name: case-kowalczyk-fleet-locked-out-2025-11
description: November 2025, 38 Kowalczyk Logistik drivers logged out at once when app 4.5 shipped the one-device rule to a fleet sharing phones
type: project
status: active
verified: 2026-02-10
---

# Case: Kowalczyk Logistik, a whole fleet logged out

Fictional Polish carrier, about 120 trucks, one of our largest carriers by volume. Ticket 2025-11-26, Enterprise plan, category `driver:login`, escalated to L3 within the hour because more than 10 drivers were affected.

## What happened

App 4.5 introduced the single-device rule: logging in on a second device revokes the first device's tokens. Kowalczyk operated with pool phones: a driver takes any phone from the shelf, logs in with his number and PIN, drives, brings it back. Several drivers had also been logged in on their personal phones. Within a morning of the 4.5 rollout, every login on a pool phone kicked a personal phone, and every login on a personal phone kicked a pool phone. 38 drivers saw the login screen during the day, some in the middle of a delivery, with pending events in their outbox.

The dispatcher called it "your update deleted our drivers".

## What we did

- L3 confirmed within 20 minutes that no data was lost: the outbox survives a logout, it is pushed on the next successful login of the same driver on the same device. Nine drivers had pending deliveries that went through once they logged back in.

- We explained the rule and did not disable it (there is no flag to disable it per org, and we did not want one). We proposed a way of working: one phone per driver, personal or pool, not both. Kowalczyk chose pool phones and bound each phone to a driver with a label.

- The release notes of 4.5 said "improved security for driver accounts". They now say what the rule does, in the three languages, and the carrier newsletter announced it two weeks before 4.6 for the carriers who had not read the notes.

- HF-3052: the app shows why you were logged out ("Your account was used on another device at 09:42") instead of a bare login screen. Shipped in 4.6.

## What we learned

- A security rule that changes how people work is a product change, not a patch. Enterprise carriers get a heads-up now.

- The support playbook had nothing about "second device". [[case-lessons-recurring-themes-2026-h1]] counts this as the origin of step 5 of the driver login playbook.

- The dispatcher's first fear was data loss. The first sentence of our answer must address that fear, before the explanation. It is in the macro now.

## Follow-up

Kowalczyk's `driver:login` tickets went from 14 in November to 2 in December and 0 in January. They later asked for the SMS PIN link to be available from their back-office, which became HF-3155.
