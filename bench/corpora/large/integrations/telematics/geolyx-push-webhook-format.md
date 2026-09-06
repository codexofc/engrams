---
name: geolyx-push-webhook-format
description: Geolyx POSTs HMAC-SHA256 signed JSON batches of up to 500 positions with UTC epoch millis to /telematics/geolyx/positions, retried 24 h with the same batch_id
type: reference
status: active
verified: 2026-04-14
---

# Geolyx push format

Geolyx is the fleet telematics platform mid-size carriers already use. Instead of polling, they push to us. Adapter: `GeolyxWebhookController` and `GeolyxBatchHandler` in `hf-telematics-gw`.

## Setup per carrier

The carrier connects their Geolyx account from `/settings/telematics`: they paste an **integration token** issued by Geolyx to their account, we call Geolyx's `POST /partners/subscriptions` with it, and Geolyx replies with a `subscription_id` and a **signing secret** for that subscription. We store both (`telematics_subscriptions`, secret hashed and also encrypted for verification, since HMAC needs the raw value). The carrier's Geolyx vehicles arrive in the next batch and appear in the mapping screen ([[tracker-vehicle-mapping]]).

The endpoint they push to is `https://telematics.halden.example/telematics/geolyx/positions`. One URL for all carriers; the subscription id in the payload tells us which one.

## The request

```
POST /telematics/geolyx/positions
X-Geolyx-Signature: sha256=<hex hmac of body>
X-Geolyx-Batch-Id: 7f3a...
Content-Type: application/json
```

```json
{
  "subscription_id": "sub_01H...",
  "batch_id": "7f3a...",
  "sent_at": 1718100000123,
  "positions": [
    {
      "vehicle_ref": "GLX-4471",
      "ts": 1718099998000,
      "lat": 50.850346,
      "lon": 4.351721,
      "speed_kmh": 62,
      "heading": 187,
      "hdop": 0.9,
      "ignition": true,
      "odometer_m": 481233000
    }
  ]
}
```

- `ts` and `sent_at` are **UTC epoch milliseconds**, from Geolyx's server, which is what we want (contrast with Trakko, see [[trakko-api-contract-quirks]]).

- Up to 500 positions per batch, from many vehicles of the same subscription.

- `hdop` is the horizontal dilution of precision; we convert to metres as `hdop * 5` for the accuracy field, which is the usual rough rule and matches what we measured.

- `vehicle_ref` is Geolyx's identifier, stable per vehicle, not the plate.

## Verification

`hash_equals(hash_hmac('sha256', $rawBody, $secret), $header)` with the secret of the subscription named in the body. Signature mismatch: 401, logged with the subscription id, counted; more than 10 in an hour for one subscription raises a ticket (the carrier may have rotated the secret on their side without telling us).

The body is read raw before any JSON decoding for the HMAC; a middleware that pretty-printed JSON in staging broke this once in 2025, which is why the controller reads `php://input` itself.

## Retries and idempotency

Geolyx retries a batch that did not get a 2xx for **24 hours** with exponential back-off, with the **same `batch_id`**. We answer 200 as soon as the batch is persisted to the inbound queue, before processing; processing failures are ours to handle. `batch_id` is stored in `telematics_inbound_batches` with a unique index, so a retried batch after a late 200 is dropped at the door (`409` to them, they treat it as success).

What happens when Geolyx has an outage and then sends everything at once is the subject of [[incident-2025-11-geolyx-duplicate-flood]].

## Rate

Per subscription, Geolyx sends about one batch per 10 seconds while vehicles move. Our ingress limit for the endpoint is 200 requests per second in total, which is 20 times the normal load, and the batch handler runs on 4 workers with a queue between the controller and the handler so a burst is absorbed, not refused.

## Things they do not send

Driver identity (good, we do not want it), fuel, temperature (some carriers asked; Geolyx has it, we do not subscribe). Ignition and odometer are the only non-position fields we keep.
