---
name: playbook-driver-app-not-syncing
description: Driver app not syncing: read sync-status, tell apart no network, poison outbox event, expired refresh token, battery optimisation
type: reference
status: active
verified: 2026-07-15
---

# Driver app does not sync

Category `driver:sync`. Symptoms as reported: "the driver pressed Delivered and the shipper sees nothing", "the load does not appear on his phone", "the sync icon spins". The server-side view is `hfctl driver sync-status <driver_id>`.

## Reading sync-status

```
last_seen_at      2026-06-12T14:02:11Z   # last authenticated request of any kind
last_push_at      2026-06-12T09:40:03Z   # last successful outbox push
outbox_depth      3                      # as reported by the app on its last request
outbox_oldest_at  2026-06-12T09:41:50Z
app_version       4.8.2
device_os         android 14
refresh_expires   2026-07-01T09:12:00Z
```

`outbox_depth` is what the app told us, not what we know. If `last_seen_at` is old, the figure is old too.

## Cases

**No contact for hours (`last_seen_at` old).** The phone has no network, is off, or the app is killed. Nothing to do on our side. Macro `driver-open-app-sync`. If it is a Samsung and the driver says the app "closes by itself", it is battery optimisation, the macro has the settings path.

**Recent `last_seen_at`, old `last_push_at`, `outbox_depth > 0`.** The app talks to us but the outbox does not drain. Almost always a poison event: the first item in the queue is rejected by the server (409 or 422) and the app retries it forever. Check the logs in Grafana with `driver_id` for the last hour, filter `route = sync_push`, read the error code. Known ones:

- `load_not_assigned`: the driver was unassigned from the load after pressing pickup. L2 clears the event with `hfctl driver outbox-drop <driver_id> <event_id> --apply` (L2 only, the event is stored in the ticket first).

- `transition_not_allowed`: two deliver events for the same load (double tap, fixed in 4.7, still seen on old versions). Same fix.

- `document_too_large`: a POD over 25 MB. The driver must delete it in the app and take a new photo, the app enforces the limit since 4.8.

**`refresh_expires` in the past.** The driver has not been online for 30 days, the refresh token is dead, the app will show the login screen on next network. Nothing to fix, tell the carrier the driver must log in again ([[playbook-driver-cannot-log-in]] if that fails).

**`app_version` below 4.6.** Update required, sync is refused with `unsupported_client`. The carrier updates the phone.

**Load not visible on the phone.** Different problem: check `driver_visible_at` in `hfctl load get`. See [[playbook-load-stuck-dispatched-no-pickup]] step 5.

## Do not

Do not ask the driver to reinstall the app as a first step. Reinstalling deletes the local outbox, and with it the delivery confirmation and the POD that were waiting. Reinstall only when the outbox is empty or L2 has copied the events.

## Escalate

L2 for any `outbox-drop`, for an error code not in the list above, and for more than 3 drivers of the same carrier in the same state within a day (that is a release problem, not a driver problem, and the backend wants to know before the triage).
