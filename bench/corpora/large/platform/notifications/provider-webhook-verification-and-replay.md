---
name: provider-webhook-verification-and-replay
description: Courrix and Bipline webhooks are HMAC-verified, deduplicated on provider event id, stored raw before being applied, and can be replayed from notification_provider_events or refetched from the provider for a time range
type: reference
status: active
verified: 2026-02-24
---

# Provider webhooks: verification, storage, replay

Two endpoints, `POST /webhooks/courrix` and `POST /webhooks/bipline`, both in `src/Notifications/Webhook/`. They are the only way delivery status moves past `sent`, so they are treated as an ingestion path, not as a callback.

## Verification

- Courrix signs the raw body with HMAC-SHA256 and puts the hex in `X-Courrix-Signature`, plus a `X-Courrix-Timestamp`. We recompute over `timestamp + "." + body`, compare in constant time, and reject anything older than 5 minutes. The key is in the vault (`notifications/courrix/webhook_secret`), rotated in January 2026 without downtime because the verifier accepts two keys during a rotation window (`COURRIX_WEBHOOK_SECRET_PREVIOUS`).

- Bipline signs the same way with `X-Bipline-Signature`, no timestamp, so the replay protection is the event id dedup below.

- Both endpoints are outside the API's session authentication and inside the ingress rate limit (300 requests a minute per source IP, allow-listed to the providers' published ranges). A request from elsewhere gets a 404, not a 401, no need to confirm the endpoint exists.

Failed verification is logged with the source IP and counted (`notify.webhook_rejected{provider}`); more than 20 in 10 minutes pages, it has fired once, when Courrix rotated their side of a key a day early.

## Storage before processing

The handler does three things and nothing else: verify, insert the raw event into `notification_provider_events(provider, provider_event_id, received_at, payload JSONB, applied_at NULL)` with `ON CONFLICT (provider, provider_event_id) DO NOTHING`, answer 202. Processing happens in a Messenger consumer (`ApplyProviderEvent`) that reads the row, maps it to a delivery through `provider_ref`, updates the status, runs the suppression policy ([[bounce-handling-and-suppression]]), sets `applied_at`.

Why not process inline: Courrix retries a webhook that does not answer 2xx within 10 s, and when the database was slow in December 2025 the inline version timed out, Courrix retried, and every event was applied twice. Dedup on `provider_event_id` fixed the double application, and moving the work out of the request fixed the timeouts. Median handler time is now 4 ms.

Events whose `provider_ref` matches no delivery (there are some, from the staging account's test sends pointed at the prod webhook by mistake in 2025) are kept with `applied_at = NULL` and `apply_error = 'unknown_delivery'`, and purged at 90 days with the rest.

## Replay

Two cases.

1. **Reapply what we have.** After a bug in `ProviderEventApplier` (once, HF-4162: soft bounces counted as hard for a day), fix the code, then `bin/console notifications:webhooks reapply --provider courrix --from 2026-02-11T00:00 --to 2026-02-12T00:00`. It resets `applied_at` on the rows in range and republishes them. The applier is idempotent on status transitions (a `delivered` after a `delivered` is a no-op, a `bounced` after a `delivered` is kept as `bounced` because the provider knows better). 180 000 events reapplied in 6 minutes that day.

2. **Refetch from the provider.** If we lost events (the ingress was down for 40 minutes in March 2026 during a certificate mishap and Courrix gave up retrying after 24 h on some), `notifications:webhooks refetch --provider courrix --from ... --to ...` calls Courrix's events API (`GET /v1/events?from=&to=&cursor=`) and inserts what is missing, same dedup, same consumer. Bipline has no events API; for SMS the fallback is `notifications:webhooks poll-status --older-than 2h` which asks `GET /v2/messages/{id}` for every delivery still in `sent` after two hours. It runs every hour anyway as a safety net, and finds 20 to 50 messages a day that never got their webhook.

## Monitoring

- `notify.webhook_events{provider, type}` per minute. A flat zero for Courrix for 10 minutes during business hours pages; it has been the earliest signal of a provider incident twice.

- Deliveries stuck in `sent` for more than 6 hours (e-mail) or 8 hours (SMS, validity 6 h plus margin): a nightly count, `warn` above 500. Normal is 100 to 200, mostly greylisting mailboxes that never send a final status.

## What is not done

We do not send webhooks to anyone. Shippers who want delivery status for their own integrations read `GET /v1/notifications/{id}` on the public API. A webhook fan-out to customers is a product someone has asked for twice and nobody has needed enough to write.
