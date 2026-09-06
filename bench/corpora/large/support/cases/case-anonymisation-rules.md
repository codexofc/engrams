---
name: case-anonymisation-rules
description: How support cases are anonymised: fictional company names from a fixed list, no people, rounded figures, dates to the month, what never appears
type: user
status: active
verified: 2026-05-01
---

# Writing a case note: anonymisation rules

Case notes are internal but they are read widely (product, backend, sales, new hires) and are copied into tickets and slides. They must be useful and must not identify a customer. These rules apply to every note in this collection.

## Names

- Customers get a **fictional company name** from the list kept in the support wiki (`case-names.md`, about 60 names, one per real customer, assigned once and reused so that two notes about the same customer are recognisably about the same one). Carriers: Transports Vauclair, Kowalczyk Logistik, Brenner Spedition, Aldemar Cargo, Rutten & Zonen, Ferreira Transportes, Nordvik Frakt, Delacroix Fret, Petrov Trans, Marchetti Autotrasporti, Bosque Logistica, Havelka Doprava, Oyelaran Haulage, Sandoval Cargo, and so on. Shippers: Meridian Agro, Boulanger Industries, Tessalia Textiles, Nordwerk Stahl, Ravello Beverages, and so on.

- The fictional name keeps the country and rough size of the real one, because that is often part of the lesson (a Starter carrier with one truck behaves differently from an Enterprise shipper).

- **No person is named**, real or fictional. "The owner", "a dispatcher", "their accountant", "their developer". No initials.

- Partners we integrate with (Cargolink, Fretzone) and vendors (Payla, Verifid) are named as they are, they are not customers.

## Identifiers and figures

- No real `org_id`, `load_id`, `invoice_id`, ticket number from Deskline, e-mail, phone number, IBAN, or customer reference. Our HF ticket keys are fine and encouraged.

- Invoice numbers, when the story needs them, are made up in the right format (`F-2025-11-0412`).

- Amounts and counts are **rounded** to two significant figures unless the exact number is the point (1.37 PLN in the rounding case is the point). Percentages and durations are kept, they are the lesson.

- Dates: month and year always; the day only when the timeline is the lesson (the Rutten case needs the 14th and the 16th).

## Content

- Quote what the customer said only in paraphrase or translated, never verbatim from a ticket, and never anything that reads as a personal remark about a customer's employee.

- Do not write what the customer did wrong as a judgement. Write what happened and what we changed. The Meridian note says "the scope of a request is the writer's", not "the customer wrote a careless e-mail".

- Every case ends with what changed on our side. A case where nothing changed is not a case, it is a ticket.

## Where they live and who checks

Notes are written by the agent who handled the case, within two weeks, reviewed by the support lead for these rules before merging. The yearly synthesis ([[case-lessons-recurring-themes-2026-h1]]) links to them. A note found with a real identifier is fixed on the spot and the author is told, once, kindly.
