---
name: oncall-prefers-chat-alerts
description: The on-call engineers want warn alerts in a single chat channel per team with resolved notifications, pages by phone only, no e-mail alerts at all, and a daily digest instead of repeated warns for the same problem
type: user
status: active
verified: 2026-02-03
---

# What the on-call people asked for

Collected from the on-call retrospectives of late 2025 and applied to [[alertmanager-routing-oncall]]. These preferences are why the routing looks the way it does.

- **No e-mail, ever.** E-mail alerts were read the next morning at best and filtered into a folder at worst. The e-mail receiver was removed in the v1 to v2 rewrite of the rules (see [[alerting-rules-catalogue]]) and nobody asked for it back.

- **One channel per team for `warn`**, not one per component. Three people asked for per-component channels in 2024 and then muted all of them. A single channel that everyone on the team reads during the day, with the alert name in bold and the dashboard link, is what gets looked at.

- **Resolved notifications for `warn`**, because a warn that fires and resolves in 10 minutes without anyone acting is information (something was briefly wrong) and it should not require a click to find out it is over. Not for `page`: the on-call closes it by hand after checking, and an automatic "resolved" on a page was once trusted when the problem had merely moved.

- **Pages by phone call, then push, then SMS**, in that order, from the paging service. Push alone was missed twice by people asleep with the phone on silent. The call cuts through silent mode on both platforms.

- **Repeated `warn` for the same problem should collapse into a daily digest**, not fire every 24 h forever. This is the one preference not fully implemented: Alertmanager has `repeat_interval: 24h` on warns, and a small script posts a morning digest of everything still firing, which is close enough that nobody complained since.

- **The summary line must contain the value and the instance**. "API latency high" is useless at 3 in the morning; "API p99 latency is 4.2s on route api_loads_search for 12m" tells the on-call whether to open the laptop or wait one more cycle. This became a rule for the `summary` annotation.

- **A runbook link on every page**, and the runbook's first section is "what to do in the first 5 minutes", not the architecture. Runbooks were rewritten in that order in November 2025.

- **No alert during a declared maintenance window** for the component being worked on. The silence is created from the runbook step, not from memory.

What they explicitly do not want: an alert for every deploy (the annotation on the dashboard is enough), alerts on CPU or memory percentages without a user-facing symptom, and any alert whose runbook says "investigate", which is not an action.
