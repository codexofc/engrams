---
name: incident-2026-06-timezone-shift-status-history
description: June 2026, a dispatch tool release wrote status changed_at in local time labelled as UTC for 2 hours, 3 100 transitions shifted by 1 or 2 hours in core.load_status_history, caught in 25 minutes by lsh_changed_at_monotonic (written for a different reason), fixed by a targeted replay, HF-4560
type: project
status: active
verified: 2026-07-02
---

# Caught: status history shifted by the local offset, 2026-06-09

## What the rule saw

At 10:40 UTC on 2026-06-09, `lsh_changed_at_monotonic` ([[dq-rule-catalog-core]]) went to `fail`: 210 loads whose latest transition in `core.load_status_history` had a `changed_at` earlier than the previous transition by more than one hour. The rule is `warn`, the dispatch channel got the message at 10:45 with 20 sample loads, and the on-duty dispatch engineer opened one: a load that went `loading` at 09:32 UTC and `in_transit` at 08:41 UTC. A truck cannot leave before it is loaded, and the second timestamp was one hour behind what the driver app's own event said.

## What had happened

The dispatch tool's release 12.4.0 at 09:00 UTC changed the status update form. The new code took the browser's local time, formatted it without an offset, and the API parsed the offset-less string as UTC (its documented behaviour for legacy clients). A dispatcher in Warsaw (UTC+2 in June) setting a status at 10:41 local produced `changed_at = 10:41 UTC` instead of `08:41 UTC`: two hours in the future. That is the opposite direction from what the rule reported, and it took ten minutes to see that the rule was catching the transitions that came *after* the bad ones: the next correct transition (from the driver app, with a proper offset) looked earlier than a bad one that sat 2 hours ahead.

The rule `lsh_changed_at_not_future` (`page`, `changed_at <= now() + 5 min`) should have fired first and did not. It runs after each `core.load_status_history` model run, every 15 minutes, and a transition written 2 hours ahead at 08:41 UTC is only "in the future" until 10:41 UTC. Most of the bad rows were evaluated within the window and should have tripped it; the rows written in the last 5 minutes before each run were, but the rule's tolerance of 5 minutes combined with the model's own 15-minute cadence meant the check saw the rows on average 7 minutes after write, and at 2 hours ahead they were still in the future. They were caught, and the alert was deduplicated into the one from 09:15, which the on-call had acknowledged as "clock skew on one workstation" because it showed 3 rows. A 3-row `page` at 09:15 looked like noise; a 210-row `warn` at 10:45 with the monotonic rule's sample loads did not. The `monotonic` rule, written in March for a different case (a replay that had re-ordered a partition), caught it because it compares transitions to each other, not to the clock.

## Response

- 10:55: cause identified from the sample loads (all had a transition from the dispatch tool user agent `dispatch-web/12.4.0`, none from the driver app).

- 11:05: release 12.4.0 rolled back to 12.3.2. New bad transitions stop.

- 11:10: scope from `raw.cdc_app_load_status_history`: 3 100 transitions between 09:00 and 11:05 from `dispatch-web/12.4.0`, from dispatchers in UTC+1 and UTC+2 (nobody in the UK that morning), shifted by 1 or 2 hours.

- 11:30 to 13:00: the source of truth is PostgreSQL, and it was wrong too (the API had stored the parsed value). Fix in PostgreSQL by the dispatch team: `UPDATE load_status_history SET changed_at = changed_at - <offset of the dispatcher's org timezone>` for the 3 100 rows, in a transaction with a before-image saved to a side table. The CDC connector emitted 3 100 `op: u` messages, `core.load_status_history` re-projected, the rule went green at 13:12.

- 12.4.1 with the offset in the payload, deployed 06-10, and the API now refuses an offset-less timestamp from any client with a user agent newer than 2025 (`400 timestamp_without_offset`), legacy clients still accepted for another quarter.

## Impact

3 100 of about 26 000 transitions that morning, on 1 900 loads. For two hours, ETA alerts on those loads were computed from shifted timestamps: 40 `load.eta_alert` notifications sent for loads that were on time, 12 not sent for loads that were late. Six support tickets. The dispatch KPIs of the day were recomputed after the fix. No invoice affected (invoicing reads the delivery date, and no delivery transition was among the 3 100, the incident being in the morning).

## What changed

- `lsh_changed_at_not_future` is now evaluated against the message's `produced-at` header carried into `raw` (the time the change was produced, not the time the rule runs): `changed_at <= produced_at + 5 min`. A 2-hour future timestamp is caught at the next model run whatever the delay. Renamed `lsh_changed_at_not_after_produced`.

- A `schema`-kind rule on the API side of things is not ours, but the data team asked for and got the API rejecting offset-less timestamps, which removes the class.

- The `monotonic` rule went from `warn` to `page`, because it caught a real incident in 25 minutes and its history since March is 2 true alerts, 0 false. This was decided at the [[dq-rules-review-feedback]] session of July.

- The timezone convention of the warehouse gained a line: "a rule that compares a business timestamp to the wall clock must state which clock, and it is never the rule's execution time".

## Numbers

| | Value |
|---|---|
| shifted transitions | 3 100 |
| loads affected | 1 900 |
| time from first bad write to rule alert | 1 h 45 (09:00 to 10:45; the rule needed the next correct transition to exist) |
| time from alert to rollback | 20 min |
| time from alert to data fixed | 2 h 27 |
| wrong ETA notifications | 40 sent, 12 missed |
| support tickets | 6 |
