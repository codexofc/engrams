---
name: incident-2026-06-webhook-secret-rotation-gap
description: On 2026-06-10 a webhook secret rotation without overlap broke 220 endpoints' verification for 40 min and 3 customers fell back to unsigned processing
type: project
status: active
verified: 2026-07-01
---

# Webhook signing secret rotation without overlap (June 2026)

Post-mortem `2026-06-10-webhook-secret-rotation.md`. **SEV2**: not a breach of our systems, but three customers' integrations accepted unsigned payloads for 40 minutes as a consequence of our rotation, which is a security regression we caused on their side.

## Background

Outbound webhooks (load events, bid events, invoice events) are signed with HMAC-SHA256 over the body, header `X-HF-Signature`. Each customer has one signing secret, shown once at webhook creation (same "shown once" rule as API keys in the IAM project). The security conventions require yearly rotation of long-lived secrets; HF-2166 implemented a scheduled rotation for webhook secrets, first run 2026-06-10 for the 220 endpoints whose secret was more than a year old.

## Timeline (UTC)

- **2026-06-10 06:00** `[job]`: `webhooks:rotate-secrets --older-than 365d` runs. For each endpoint: generate a new secret, store it, email the technical contact "your webhook secret has been rotated, retrieve the new value in settings". Deliveries from 06:00 are signed with the new secret. No overlap: the old secret is gone.

- **06:00 to 06:40** `[log]`: 4 100 webhook deliveries to 220 endpoints signed with secrets the customers do not have yet. 180 endpoints return 401 or 400 (signature check failed); our retry logic queues them. 37 endpoints return 200: those customers do not verify signatures at all (known, and their problem). **3 endpoints return 200 after first returning 401**: their code falls back to accepting the payload when verification fails and logging a warning. We learn this from their support tickets later that morning.

- **06:20** `[chat]`: first customer message "all our webhooks fail signature since 06:00". Incident opened 06:32, SEV2 once the fallback behaviour is understood at 08:15 (started as SEV3).

- **06:40**: containment: rotation reverted for all 220 endpoints (the old secrets were kept in `webhook_endpoints.previous_secret_hash`, encrypted, for exactly this kind of rollback; the job's author had planned for revert but not for overlap). Deliveries from 06:40 are signed with the old secrets again. Queued retries drain by 07:30.

- **08:15**: the three fallback customers are called. Two turn off the fallback the same day; the third says it is deliberate ("we would rather process than lose events") and is told in writing that we consider unsigned processing unsafe.

- **09:00**: incident closed. Root cause and the rotation redesign go to the post-mortem.

## Root cause

The rotation design assumed the customer would retrieve the new secret between rotation and the next delivery, which for busy endpoints is seconds. A rotation without an overlap window is, from the customer's side, an outage of the signature check, and some integrations respond to an outage by disabling the check.

## What went well

- `previous_secret_hash` existed, so the revert took one command.

- The post-mortem found the three fallback integrations, which were a latent problem independent of our rotation.

## Actions

- **Overlap window** (HF-2192, done 2026-06-24): a rotation creates the new secret and keeps the old one valid for **7 days**. During the window every delivery carries two signatures: `X-HF-Signature: v1=<hmac old>,v1=<hmac new>`, plus `X-HF-Signature-Kid: <id old>,<id new>`. A verifier that checks any one of the listed signatures passes. This is the same idea as the API key rotation overlap in the IAM project, applied late.

- **Customer-triggered rotation** from settings, with the same overlap, so a customer who suspects a leak does not wait for us.

- **Rotation announced 14 days ahead** by email with the exact date, and the new secret is retrievable during those 14 days (marked "pending"), so a customer can deploy it before the switch.

- **Documentation** on `docs.halden.example/webhooks/signing` gained a section "never fall back to unsigned processing; on verification failure, return 401 and let us retry", and the three fallback customers received it directly.

- The next scheduled rotation ran 2026-07-08 with the overlap: 2 tickets, both from customers who had ignored the announcement, both fixed within the 7 days.

Ticket: HF-2191.
