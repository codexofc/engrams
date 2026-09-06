---
name: incident-timeline-tooling
description: The scribe records the timeline with /t in chat, stamped UTC into incident_timeline; /t export builds the post-mortem table and merges audit_events
type: reference
status: active
verified: 2026-04-12
---

# Timeline tooling for the scribe

Built in HF-2110 alongside the incident process ([[incident-process-severity-levels]]), after reconstructing two 2025 timelines from chat history with hour-wide gaps.

## Recording

In an incident channel `#inc-HF-xxxx`, the scribe (or anyone) types:

```
/t [source] free text
```

Examples:

```
/t [chat] customer IT reports POD link from 01-14 still opens
/t [audit] auth.login_succeeded staff account from unknown country, ip prefix 203.0.113.0/24
/t [action] rotated storage signing key, all presigned urls invalid
/t [decision] SEV raised to 1, commander: security rota
```

The bot stores each line in `incident_timeline (incident_ticket, recorded_at, recorded_by, source, text)` with `recorded_at` in UTC from the server clock, not the client's. It echoes the line back with the stamp so the channel shows it. `source` must be one of `chat`, `audit`, `log`, `cloud`, `forge`, `vault`, `ticket`, `job`, `action`, `decision`, `memory`; anything else is rejected with the list.

Backdating: `/t @14:10 [log] first anonymous ListBucket request` records `recorded_at = now` and `event_at = 14:10 UTC today`; the export shows `event_at` with a marker that it was entered later. Yesterday: `@2026-02-12T14:10`.

## Exporting

`/t export` posts a markdown table (event time, source, text) sorted by event time, ready to paste into the post-mortem ([[postmortem-template-rules]]). Lines with `source = memory` are rendered in italics with `[memory]`, and backdated lines carry `(entered HH:MM)`.

`/t export --with-audit <user_id|org_id> <from> <to>` merges `audit_events` rows for that actor or organization in the window, one line each, `source = audit`, so the scribe does not retype them. It was the single most useful addition: the impersonation activity in the March drill ([[security-incident-drill-2026-03]]) and the attacker's 41 read requests in the phishing case would both have been one command.

## Rules that the tool enforces

- No edits. A wrong line is followed by `/t [decision] previous line wrong, correct value is X`. The record of the mistake is part of the timeline.

- No deletion except by the commander with `/t retract <id> reason`, which keeps the line in the table with strike-through and the reason. Used once (a line contained a customer's full name).

- Every `/t` line is also written to the incident ticket as a comment every 15 minutes in batch, so someone without chat access (the DPO on the phone) can read the ticket.

## What it does not do

- It does not page anyone, and it does not change severity. Those are commands of the `/incident` bot, and the scribe records their effect with `/t [decision]`.

- It does not replace the post-mortem narrative. The export is the source for the timeline section; the other six sections are written by a person.

- It is not a general logging tool. A line that says "looked at grafana" is noise; "grafana: 4xx on /v2/documents from 14:10, 200 rps, normal is 20" is signal. The scribe guide in `halden-security/scribe.md` has ten examples of each.

## Storage

`incident_timeline` is kept forever (small: 2 400 lines for 17 incidents and drills in April 2026). It contains no personal data by rule; the one retraction is the exception that confirmed the rule is needed.
