---
name: webhook-integrator-feedback-idempotency
description: March 2026 integrator survey (31 answers): they want a delivery id in the body and JSON schemas, do not want batching or push-to-queue
type: feedback
status: active
verified: 2026-04-22
---

# Integrator feedback, March 2026

We asked every organisation with an active subscription (about 180) to answer eight questions about the webhooks. 31 answered, mostly the technical contact, half of them external integrators working for our customers. Not representative of the silent majority, but the silent majority does not open tickets either.

## What works for them

- The signature scheme. Nobody asked for something else. Three said the guide's snippets were the reason it worked first time.

- The delivery viewer in the web screen with `last_error` in words. "The first webhook provider where I can see what my server answered without asking support."

- Replays from the screen (new in April, some answered after it shipped). One said they use it as a poor man's test suite: change code, replay yesterday, compare.

## What they want

**A stable event id in the body.** They dedupe on `X-Halden-Delivery` as told, but several store payloads in a queue where headers are lost, and they end up wrapping the body themselves. In v3 `meta.delivery_id` is in the body; four v2 integrators asked for the same in v2. Decision: added `delivery_id` to v2 bodies as a new field (additive, allowed in v2), HF-3122, shipped April.

**The JSON schemas.** Eleven asked. Published per version and per event since the June incident made it urgent ([[webhook-incident-2026-06-payload-v3-null-driver]]).

**Ordering.** Six asked for guaranteed order. Answered with the [[webhook-ordering-and-sequence-numbers]] note's content. Two were satisfied by `sequence`, two by the "fetch the load" advice, two would like order anyway. No.

**Filters.** Five asked to receive only events for loads in a country or of a visibility. Shipped as subscription `filters` in HF-3108, two fields to start.

**Longer response timeout.** Three asked for more than 10 seconds because they do synchronous work in the handler. Answered no: acknowledge, then work; the guide has a paragraph. One replied that "everyone says that and everyone does it synchronously anyway".

## What they do not want

- **Batching** (several events per request). We asked because we had considered it for high-volume shippers. 24 of 31 said no; the handling code is simpler per event and their queues take one message per event anyway. Dropped from the roadmap.

- **A push-to-queue option** (we publish directly into their cloud queue instead of HTTP). Two asked, the rest did not care, and the credential handling would be a new kind of secret to store. Not doing it.

## Quotes worth keeping

- "Your retries are too patient. If my server is down for two days, I will replay myself, I do not need you to keep trying." (A minority view; the v2 retry schedule went the other way because the data said most outages are weekends.)

- "Please never change the shape of a field inside a version." (Written in March. We did it in June. Now in the process.)

- "The 5-minute clock skew rule killed us for a day; put it in red." It is in red.

## Method

Eight questions, free text allowed, sent by e-mail to the technical contact of each subscription, two reminders. Next round planned for March 2027, same questions plus one about v3 adoption.
