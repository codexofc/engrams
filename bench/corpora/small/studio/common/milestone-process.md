---
name: milestone-process
description: Marée milestones are 6-week cycles ending in a playtest build, scope freeze at week 4, bug-only week 6, Monday review
type: project
status: active
verified: 2026-04-02
---

## Cycle

Six weeks per milestone since September 2025 (before that, quarterly, which produced two death marches).

- Week 1: the three goals of the cycle are written in the milestone ticket (`BR-M<nn>`), each with a measurable end state ("the boat can dock at the three harbours of the north bay", "60 fps on `port_nuit` on the low-end target").
- Weeks 1 to 4: work. Every merge request references a goal or a bug.
- Week 4, Friday: scope freeze. Anything not merged that is not a bug moves to the next cycle. No exception without the producer and the lead of the area agreeing in the ticket.
- Week 5: integration and content polish; the assets lane runs a full rebuild.
- Week 6: bugs only, ordered by [[bug-severity-scale]]. Tag `playtest/M<nn>` on Thursday, internal playtest Friday, external playtest the following week when there is one.

## Review

The Monday after the playtest, one hour, everyone. Three questions: were the goals met (yes or no, with the measurement), what did the playtest say ([[playtest-feedback-loop]]), what are the three goals of the next cycle. The producer writes the result in the milestone ticket the same day.

## Numbers

M12 (February to March 2026): 3 goals, 2 met, 214 merge requests, 61 bugs fixed in week 6, 9 open at tag time (all severity 3 or 4). M13: 3 of 3 met, the first time, with 48 bugs in week 6.

## What was dropped

- Daily stand-ups. Replaced by a written check-in in the channel before 10:00; the engine and tools teams found the meeting cost more than it gave.
- Estimates in days. Goals are sized so that the lead of the area thinks they fit in 4 weeks, and the freeze does the rest.
