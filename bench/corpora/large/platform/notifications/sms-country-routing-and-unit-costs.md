---
name: sms-country-routing-and-unit-costs
description: SMS routing per destination country (sender id, validity, OTP allowed or not) and Bipline unit costs, 0.034 to 0.11 EUR, Germany and Belgium force a long number, monthly SMS bill about 11 000 EUR
type: reference
status: active
verified: 2026-06-02
---

# SMS routing by country and what a message costs

The routing table is `config/notifications/sms_routing.yaml`, loaded by `SmsRouter::route(PhoneNumber $to, string $eventType)`. It decides the sender id, the validity period, and whether the message may be sent at all for this event type. The provider side is in [[sms-provider-bipline-integration]].

## Why a table exists

Three reasons, learned one at a time.

1. Alphanumeric sender ids (`HALDEN`) are filtered or rewritten in some countries. Germany rewrites them to a random long number since 2024 unless pre-registered, Belgium blocks unregistered ones, and Hungary requires a registration we have not done. A driver in Germany receiving an OTP from an unknown number does not trust it, and the support ticket says "is this you".

2. OTP over SMS is the wrong tool where delivery is slow. Bipline's median delivery in Romania is 9 s, in Italy it was 40 s in 2025 on one operator, and an OTP that arrives after the 5-minute window has been paid for nothing. Countries with slow delivery get a shorter validity so the message is dropped instead of arriving stale.

3. Unit costs vary by a factor of three, and the routing table is the only place where "should this be an SMS at all" is decided per country.

## The routing table, June 2026

### Sender id and validity by country

| Country | Sender | Validity | OTP by SMS | Notes |
|---|---|---|---|---|
| FR | `HALDEN` | 6 h | yes | |
| PL | `HALDEN` | 6 h | yes | biggest volume, 31 % |
| RO | `HALDEN` | 6 h | yes | |
| DE | long number `+49 ...` | 6 h | yes | alphanumeric rewritten by operators |
| BE | long number `+32 ...` | 6 h | yes | alphanumeric blocked |
| NL | `HALDEN` | 6 h | yes | |
| ES | `HALDEN` | 6 h | yes | |
| IT | `HALDEN` | 2 h | e-mail first | slow on one operator |
| HU | long number `+36 ...` | 6 h | yes | registration not done |
| CZ | `HALDEN` | 6 h | yes | |
| others | long number FR | 2 h | e-mail first | 2 % of volume |

The long numbers are rented from Bipline, 15 EUR a month each. "e-mail first" means the OTP goes by e-mail and SMS is the fallback after 90 s, the reverse of the default.

## Unit costs

Bipline invoices per segment, per destination country, and the rate card changed once in January 2026 (average +6 %). Figures below are per segment in EUR, excluding VAT, from the May 2026 invoice.

### Cost per segment by country

| Country | EUR / segment | Share of May volume | Share of May cost |
|---|---|---|---|
| PL | 0.034 | 31 % | 22 % |
| FR | 0.058 | 22 % | 27 % |
| RO | 0.041 | 14 % | 12 % |
| DE | 0.084 | 11 % | 19 % |
| ES | 0.052 | 7 % | 8 % |
| NL | 0.076 | 5 % | 8 % |
| IT | 0.062 | 4 % | 5 % |
| BE | 0.110 | 3 % | 7 % |
| others | 0.09 (avg) | 3 % | 6 % |

May 2026: 296 000 segments, 11 200 EUR. Germany and Belgium together are 14 % of the messages and 26 % of the money, which is the argument for pushing German drivers to the app (push is free) rather than SMS, see [[push-first-sms-fallback-decision]].

## Segments, not messages

A message in GSM-7 up to 160 characters is one segment, 161 to 306 is two, up to 459 is three. A single character outside GSM-7 turns the whole message into UCS-2, 70 characters per segment. The templates lint flags any SMS template whose longest rendering (with the longest fixture) exceeds one segment, and there are exactly four two-segment templates left, all assignment details with an address. Average segments per message: 1.12 in May, down from 1.31 in November 2025 when the lint was introduced. That 0.19 is about 1 900 EUR a month.

## What the router refuses

- An SMS to a number whose country is not in the table and whose event type is not in the `allow_unknown_country` list (OTP and assignment only). A marketing-shaped event to an unknown country is a configuration bug, not a message.

- An SMS to a number on the SMS suppression list ([[bounce-handling-and-suppression]]).

- More than the per-recipient limit ([[per-recipient-rate-limits]]), which is 5 SMS an hour and 20 a day for any recipient, any event type.

## Changing the table

The table is data, but a change is a deploy, and the change needs a Bipline-side check when a sender id is involved (they pre-register `HALDEN` per country). The January 2026 addition of Czechia took a week on their side. The `SmsRouterTest` fixtures include one number per country in the table and one unknown, and the CI fails if a country appears in `users.phone` prefixes with more than 500 users and is missing from the table (the check runs against an anonymised prefix count exported weekly, `fixtures/phone_prefix_counts.csv`).

## Monthly review

The first working day of the month, the platform team reads the Bipline invoice against `notification_deliveries` aggregated by `sms_country` and `segments`. The reconciliation has been within 0.5 % every month except February 2026, when Bipline billed 9 400 undelivered messages with `expired` status that the contract says are free; credited the following month after one e-mail.
