---
name: crash-triage-lesson
description: April 2026 playtest: 212 crashes were 3 bugs, symbolicated minidumps plus replays made triage 10 minutes, no replay means no triage
type: feedback
status: active
verified: 2026-05-04
---

## What happened

The April 2026 external playtest (40 testers, 3 days) produced 212 crash reports. Before it, crash triage meant reading a stack trace from a debug log and guessing. This time every build shipped with minidump capture, symbol files kept per build on the build server, and the input replay of the last 5 minutes attached to the report.

## What we learned

- 212 crashes were 3 bugs (170 reports), plus 11 others. Grouping by the top 3 symbolicated frames did the grouping in minutes; without symbols, the same crash looked like 20 different addresses across the 4 builds shipped that week.
- The replay ([[physics-fixed-step]] describes its determinism) reproduced 2 of the 3 big bugs on the first try on a dev machine. The third was a GPU hang that the replay could not reproduce because it depended on the tester's driver version; the report's driver string found it.
- The 11 rare crashes: 6 reproduced from the replay, 5 not. All 5 had no replay attached (the tester had restarted the game within 5 minutes, and the replay buffer was empty). We fixed the buffer to persist across restarts.

## Rules now

- A crash report without symbols or without a replay is not triaged by hand. The pipeline rejects a build for playtest if its symbols are not on the build server.
- Triage is grouped by top 3 frames first, count second. The most frequent group is fixed first, whatever its severity, because 170 reports for one bug means testers stop playing.
- The replay buffer is 5 minutes and survives a restart; a bug that needs more is reproduced with the QA replay tools, not from crash reports.

## How to apply

For any tool or system producing reports (crashes, asset failures, missing shaders), ship the grouping key and the reproduction artefact with the report, or the report is noise. The tools team applied the same idea to asset import failures the following month.
