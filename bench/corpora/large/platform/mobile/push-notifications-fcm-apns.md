---
name: push-notifications-fcm-apns
description: Push goes through the API's notifications transport to FCM (Android) and APNs (iOS) directly, data-only messages for sync triggers, alert messages for assignments, tokens refreshed on every app start and pruned after 60 days
type: reference
status: active
verified: 2026-05-06
---

# Push notifications

## Two kinds of messages

**Silent / data-only** (`type: sync`): no visible notification, the app wakes and runs a delta pull. This is the trigger in [[sync-push-triggered-delta]]. On Android, FCM data message with `priority: high`. On iOS, APNs with `content-available: 1` and `apns-priority: 5`, and the OS decides if and when to deliver, see [[ios-background-fetch-limits]].

**Alert** (`type: assignment`, `type: message`, `type: reassignment`): visible notification with title and body, localised server-side from the driver's `locale` (fr, en, pl, ro, es, de). Tapping opens the load via a deep link `halden://load/<uuid>` handled by `AppRouter`.

The server never puts business data in the alert payload beyond the load reference and the type. The app fetches the rest. Reason: the payload is visible on the lock screen and in the push providers' logs.

## Token lifecycle

- The app registers its token on every start and every token refresh callback with `PUT /internal/mobile/devices/{installation_id}` (`installation_id` is a UUID generated at first launch and stored in the keychain, survives reinstall on iOS, not on Android).

- Server table `device_tokens (installation_id, user_id, platform, token, app_version, last_seen_at)`. One row per installation, the token is overwritten.

- Tokens not seen for 60 days are deleted nightly. A token rejected by FCM with `UNREGISTERED` or by APNs with 410 is deleted immediately.

- One driver can have one active device (see the sync note), so in practice one token per user, but the schema allows more because support staff use the app on two phones.

## Delivery on the API side

`SendPushHandler` on the `notifications` Messenger transport. FCM HTTP v1 with a service account, APNs with a token-based `.p8` key, both secrets mounted from Kubernetes, never in the image. Per-message timeout 5 s. A 429 from either provider raises a `RecoverableMessageHandlingException` with the provider's retry delay.

Since HF-1640 (the invoice e-mail incident) every push has an idempotency key `push:<installation_id>:<event_id>` in `notification_deliveries`, so a retried handler never sends twice.

## Measured

- Delivery latency for alert messages: p50 1.2 s, p95 6 s (Android), p95 20 s (iOS, because APNs coalesces).

- Silent messages on Android: 92 % delivered within 30 s when the device is not in Doze. Under Doze, the high-priority data message still wakes the app but the quota is about 10 per day, which is why the 15 min timer fallback exists.

- Silent messages on iOS: 60 % delivered within 5 min, 25 % never (device in Low Power Mode or app killed by the user). Not a bug, an OS policy. This is why iOS drivers see more "pull to refresh".

## Testing

`flutter test` uses `FakeFirebaseMessaging`. A real device test is in the release checklist (see [[release-process-stores]]): assign a load from staging, check the notification arrives, tap it, check the deep link.
