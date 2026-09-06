---
name: sync-push-triggered-delta
description: Since app 4.5 (HF-1220) sync is a delta pull with a server cursor, triggered by a silent push on server-side changes, with a 15 min timer fallback, cutting sync traffic by 94 %
type: project
status: active
verified: 2026-02-12
---

# Delta sync triggered by silent push (HF-1220)

Shipped in app 4.5 and API release 2025.41, October 2025. Replaces [[sync-polling-interval-old]].

## Server side

- `GET /internal/mobile/sync?since=<cursor>` returns changed entities of the driver since the cursor and a new cursor. The cursor is `base64(updated_at of the newest row returned || its uuid)`, same idea as the public cursor pagination. A missing `since` returns the full set of the last 30 days.

- The response has `loads`, `stops`, `documents`, `messages`, `tombstones` (deleted ids with the entity kind) and `cursor`. Maximum 500 entities per call, `has_more` when truncated, the app loops.

- Every write on the API that touches a driver's data (assignment, stop edit, new message, document available) dispatches `NotifyDriverSync(driverId)` on the `notifications` transport. The handler debounces per driver with a Redis key `sync-notify:<driver>` with a 10 s TTL: the first event sends the silent push, the following ones within 10 s are dropped. A dispatcher editing five stops produces one push.

## App side

`SyncScheduler` has three triggers that all end in the same `SyncCoordinator.pull()`:

1. Silent push received (`type: sync`), see [[push-notifications-fcm-apns]].

2. App comes to the foreground.

3. Timer: 15 min when GMS is available, 5 min without ([[device-quirk-huawei-no-gms]]), only while the app process is alive.

`pull()` is serialised with a mutex, a trigger during a pull sets a `dirty` flag and a second pull runs right after. This is what prevented the retry storm from getting worse in December, see [[incident-2025-12-sync-storm-after-release]].

The cursor is stored in the `sync_state` table with the time of the last successful pull. A 422 `invalid_cursor` (server rotated its format) clears the cursor and does a full sync.

## Numbers

Two weeks before and after, October 2025:

- Sync requests per minute at peak: 1 200 → 70. That is 94 % less.

- Outgoing bytes per day for sync: about 2 GB → 110 MB.

- Time from assignment to visible on the driver's phone: p50 4 s on Android, 45 s on iOS (APNs delay), was up to 15 min.

- Silent push volume: 26 000 per day.

## What it does not solve

iOS devices in Low Power Mode do not receive silent pushes reliably. Those drivers rely on the foreground trigger. There is no fix on our side, only documentation for dispatchers ("if an iPhone driver does not see the load, ask them to open the app").

## Ordering with the outbox

A pull never runs while a push of pending mutations is in flight. The reason is in [[offline-sync-architecture]]: the replica must not be overwritten with server state that predates a mutation still in the outbox, or the projection would flicker.
