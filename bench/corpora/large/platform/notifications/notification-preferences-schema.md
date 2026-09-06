---
name: notification-preferences-schema
description: notification_preferences holds one row per user and event group with a channel mask; defaults from event_types.yaml, critical events ignore preferences
type: reference
status: active
verified: 2026-03-18
---

# Notification preferences

## Model

`notification_preferences(user_id, event_group, channels smallint, updated_at, updated_by)`, primary key `(user_id, event_group)`. `channels` is a bit mask: 1 e-mail, 2 SMS, 4 push. No row means "defaults for this group". A row with `channels = 0` means "nothing for this group". The table has 210 000 rows for 380 000 users, most users never touch their preferences.

Plus two columns on `users`: `all_email boolean default true` (the one-click unsubscribe target) and `prefers_sms boolean default false` (support-set flag for drivers who want SMS rather than push, see [[push-first-sms-fallback-decision]]).

## Event groups

Event types are grouped for the preferences UI so that a user faces eight switches, not sixty. The group is declared per event type in `config/notifications/event_types.yaml`:

| Group | Example event types | Default channels | Who sees it |
|---|---|---|---|
| `bids` | `bid.received`, `bid.digest`, `bid.withdrawn` | e-mail, push | shippers |
| `awards` | `bid.accepted`, `bid.rejected` | e-mail, push | carriers |
| `assignments` | `load.assigned`, `load.reassigned` | push (SMS fallback) | drivers, dispatchers |
| `tracking` | `load.status_changed`, `load.eta_alert` | push | shippers, dispatchers |
| `documents` | `document.available`, `document.rejected` | e-mail | all |
| `billing` | `invoice.issued`, `invoice.overdue`, `payout.sent` | e-mail | billing contacts |
| `account` | `auth.new_device`, `auth.password_changed` | e-mail | all, not switchable |
| `system` | `auth.otp`, `auth.security_alert`, `load.cancelled` (critical) | as declared | all, not switchable |

`account` and `system` are shown but greyed out. A user cannot turn off "your password was changed", and an OTP is not a preference.

## Resolution order

`ChannelResolver::resolve(User $u, string $eventType): ChannelSet`:

1. Start from the event type's `channels` in the yaml.

2. If the event is `critical`, stop here. Preferences are ignored, `all_email` is ignored, the suppression list still applies (a dead mailbox is dead).

3. Intersect with the user's row for the event group, if any.

4. If `all_email = false`, remove e-mail.

5. Remove channels the user cannot receive: SMS without a verified phone, push without a device token. The removed channel is recorded on the delivery as `skipped_reason` so `notifications:trace` can say why there was no SMS.

6. If the result is empty and the event group is not `bids` or `tracking`, fall back to e-mail if `all_email` is true. A user who turned off every channel for `billing` still gets invoices by e-mail; this was a product decision in HF-4108, recorded in the yaml as `min_channel: email`.

## API

`GET /v1/me/notification-preferences` returns the eight groups with `channels` and `available_channels` (what the user could enable given their verified contacts). `PUT` takes the same shape. Changes write `updated_by = user` or `updated_by = support:<id>` and an audit row. The dispatch tool's preferences page and the driver app both use these two endpoints; there is no other write path, the old direct SQL from the back-office was removed with HF-4108.

## Migration from the old model

The old table `user_notification_settings` had one boolean per event type per channel (60 × 3 columns, mostly null). The migration mapped each event type to its group and took the "or" of the channels within a group, on the reasoning that a user who had e-mail on for `bid.received` and off for `bid.withdrawn` wanted bid e-mails. 1 200 users had contradictory settings inside a group; they got the union and a one-time e-mail explaining the new switches. Four replied, all fine with it.

## Testing

`ChannelResolverTest` has one case per row of the resolution order and one per group, with the yaml loaded from the real file so that a new event type without a group fails the test, not production. The lint `notifications:lint-templates` also checks that every event type has a group.
