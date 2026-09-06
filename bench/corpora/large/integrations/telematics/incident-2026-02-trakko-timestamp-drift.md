---
name: incident-2026-02-trakko-timestamp-drift
description: From 2026-02-03 to 05, 61 Trakko units reported device clocks 20 to 45 min behind, causing 340 false delay alerts; fixed by trusting received_at over 120 s gap
type: project
status: active
verified: 2026-02-27
---

# Trakko device clock drift (February 2026)

Ticket HF-2115. Not a security incident; an integration incident run with the platform on-call process. Written here because the fix is in the adapter and the lesson is about trusting device time.

## Symptom

- **2026-02-03 morning**: dispatchers of 9 carriers report that the map shows their trucks "behind" where the driver says they are, and that ETAs jumped to "late" for trucks that were on time. The delay alert (`assignment.delay_detected`, sent to the carrier dispatcher when ETA exceeds the slot by 15 min) fired 340 times over three days for those carriers, against a normal 30 per day for the whole platform.

- All affected vehicles had a Trakko unit and no app in use. Vehicles with the app were fine.

## Investigation

`position_events` for one affected vehicle showed positions with `ts` 20 to 45 minutes behind the wall clock at which they were written (`ingested_at`). The raw payloads (7 day retention in `hf-telematics-raw`) had `timestamp` values that lagged `received_at` by the same amount. The device clock was wrong; Trakko's server received the positions promptly but forwarded the device's idea of time.

Sixty-one units, all from the batch installed in autumn 2024. Trakko's explanation, after two days: those units have a backup battery that keeps the real-time clock when the truck's power is cut; the batteries were failing, and on power-up the unit restored the clock from its last saved value (up to 45 minutes stale, the save interval) and then took up to a day to resync from GPS because the firmware only resyncs on a cold GPS fix.

Our plausibility stage rejected timestamps more than 24 h in the past or 60 s in the future, so a 45 minute lag passed. The ETA consumer computed from a position that was, from the vehicle's point of view, 45 minutes old, so the vehicle looked 45 minutes behind.

## Fix

- **Adapter rule** (deployed 2026-02-05 15:00 UTC): `TrakkoAdapter` reads `received_at` (present in the payload, undocumented at the time, since documented at our request). If `|timestamp - received_at| > 120 s`, the position's `ts` becomes `received_at` and `time_source = 'server'` is set on the event. 120 s because Trakko units buffer up to 90 s of positions when the cellular link drops, and that legitimate lag must not be overwritten. Details in [[trakko-api-contract-quirks]].

- **Backfill**: positions of the 61 vehicles for 02-03 to 02-05 were re-timestamped from the raw payloads (`telematics:reprocess --provider trakko --vehicles <list> --from --to`), 210 000 rows updated in place; ETAs recomputed. The false delay alerts could not be un-sent; the 9 carriers received an explanation.

- **Monitoring**: a new metric `telematics_time_gap_seconds{provider}` histogram, alert when the p50 for a provider exceeds 60 s for 10 minutes. This would have fired within the first hour on 02-03.

- **Hardware**: Trakko replaced the 61 units under warranty by end of March 2026; the replacement firmware resyncs the clock on every GPS fix.

## What we learned

- Device time is a claim, not a fact. Every adapter now records both the device and the server timestamp, and the pipeline's `ts` is the one we trust after the gap rule; the other is kept in `ts_device` for forensics.

- The plausibility window (24 h past, 60 s future) was designed for garbage, not for plausible-but-wrong. The gap rule is the second layer.

- The app was unaffected because it timestamps from the phone, whose clock is network-synced, and the mobile team already had a rule about not trusting device clocks for a different reason (their notes have it). We should have asked them a year earlier.

Related: how the harness now replays this exact scenario, in [[telematics-integration-test-harness]].
