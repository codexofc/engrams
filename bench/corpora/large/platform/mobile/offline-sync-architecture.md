---
name: offline-sync-architecture
description: Driver app keeps a local SQLite (drift) copy of the driver's loads and an append-only outbox of mutations, synced by a delta pull and an ordered push, designed for 4 h without network
type: reference
status: active
verified: 2026-04-08
---

# Offline sync in the driver app

The app must work with no network for a full delivery round. The measured worst case from telemetry is 3 h 50 min without a single successful request (a driver in the Massif Central, October 2025), so the design target is 4 h and the storage budget is 7 days.

## Local state

SQLite through drift (see [[local-db-drift-schema-migrations]]). Three kinds of tables:

- **Replicas**: `loads`, `stops`, `documents`, `messages`. Exact copies of what the server sent, plus `server_updated_at` and `synced_at`. Never modified by the UI directly.

- **Outbox**: `pending_mutations (id, seq, kind, payload json, created_at, attempts, last_error)`. Every user action that changes server state is a row here first. `seq` is a local autoincrement, and mutations are pushed in `seq` order, always.

- **Projections**: what the UI reads. A `load_view` is the replica with the pending mutations applied on top, computed in `LoadProjector`. If the driver marked a stop delivered and the push has not happened yet, the UI shows it delivered. This is the whole trick: the UI never waits for the network.

## Pull (delta)

`GET /internal/mobile/sync?since=<server cursor>` returns every load, stop, document and message of the driver changed since the cursor, plus a new cursor. The cursor is the server's `max(updated_at)` seen, opaque to the app. First sync sends no cursor and gets everything assigned in the last 30 days.

Pull is triggered by [[sync-push-triggered-delta]] (a silent push), by app foreground, and by a 15 min timer as a fallback. It replaces [[sync-polling-interval-old]].

A pull that fails is retried with exponential backoff capped at 5 min, and never blocks the UI.

## Push (ordered outbox)

`SyncPusher` takes pending mutations in `seq` order and sends them one by one to `POST /internal/mobile/mutations` with the mutation `id` as `Idempotency-Key`. The server stores processed keys for 30 days, so a retried mutation is a no-op with a 200.

Rules:

- Strictly one in flight. Parallel pushes reorder events and the server rejects a `deliver` before a `pickup`.

- A 4xx other than 409 and 429 marks the mutation `FAILED` and stops the queue. The driver sees a banner "Une action n'a pas pu être envoyée" with a retry button and support gets a log line. We chose to stop rather than skip because the next mutations usually depend on the failed one.

- A 409 is a conflict, handled by [[sync-conflict-resolution-rules]].

- 5xx and network errors: retry with backoff, queue stays.

After a successful push of all pending mutations, a pull is triggered so the replica catches up with the server-side effects (status changes, new documents).

## What is not synced

Positions (GPS) go through a separate channel, batched, lossy, see [[gps-tracking-battery-budget]]. Photos are uploaded by the document pipeline, not by the mutation outbox, see [[pod-photo-compression]]. The mutation only carries the document id once the upload completed.

## Sizes measured (app 4.8, March 2026)

- Typical local DB: 3 to 6 MB. Largest seen: 41 MB for a driver with 800 loads in 30 days (a shuttle driver). Fine.

- Outbox rarely exceeds 30 rows. The record is 214 after the [[incident-2025-12-sync-storm-after-release]].

- Full initial sync: 1.2 s on Wi-Fi, 9 s on a bad 3G connection.

## Known limitations

- No partial sync of a single load. A change on one stop re-sends the whole load (about 4 KB). Acceptable.

- Deleted loads: the server sends `deleted: true` tombstones for 30 days. A device offline longer than 30 days does a full resync and drops everything local first. Happened twice, no data lost because the outbox is pushed before the drop.

- Two devices for one driver are not supported. The second login revokes the first (refresh token family), and the first device's outbox is lost if it had pending mutations. Documented for support, product accepted it.

## Purge, storage budget and debug tools

The 7 day storage budget means: a device keeps loads terminated (delivered, cancelled, reassigned away) for 7 days after their last change, then `LocalPurge` deletes them with their stops, documents metadata and messages. Documents' cached files are deleted after 48 h regardless, since the app re-downloads them through a presigned URL when opened. Pending mutations are never purged by age: a mutation stays until pushed or explicitly dropped by the conflict resolver. The 30 day window that the server sends on a full sync is therefore larger than what the device keeps, and that is fine, the local copy is a working set, not an archive.

`LocalPurge` runs on app start if the last run is older than 24 h, and in the iOS `BGProcessingTask` when it gets one. It logs `db.purge` with the counts and the database size before and after. The telemetry event `db.size` (once a day per device) is where the 41 MB record came from, and the distribution in April 2026 was p50 4.2 MB, p95 12 MB, max 41 MB. Nothing to act on.

Debug tools in the app, behind the hidden settings menu (7 taps on the version number, which support knows):

- **Sync log**: the last 200 sync operations with timestamp, trigger (`push`, `foreground`, `timer`), result, duration, entity counts, and for failures the HTTP status and the `trace_id`. Drivers send a screenshot of it to support, and the `trace_id` makes the rest of the investigation server-side.

- **Outbox view**: pending mutations with their `seq`, kind, age and last error. A "Push now" button. And a "Drop this mutation" action that requires typing the load reference, used by support on the phone with the driver in the rare case where one mutation is stuck on a 4xx and the driver needs the rest of the queue to go through.

- **Export database**: writes a copy of the SQLite file to the share sheet, so a driver can send it to support. The file contains addresses and names of the driver's own loads only. It has been used four times, each time to reproduce a projection bug on a developer's machine, which is the fastest way to fix that kind of bug.

- **Force full resync**: clears the cursor and the replicas, keeps the outbox, pulls everything. The last resort, and it says so on the button.

None of these tools exist in the release build's normal UI, all of them exist in every release build. Removing them from release was discussed and rejected: the day they are needed is a day the driver cannot install a debug build.
