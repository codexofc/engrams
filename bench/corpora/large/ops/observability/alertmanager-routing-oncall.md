---
name: alertmanager-routing-oncall
description: Alertmanager routes by team and severity, page goes to the paging service for the team on call (platform, ops, mobile) and warn goes to the team chat channel, grouped by alertname and namespace with 30 s group_wait, repeat 4 h for page and 24 h for warn
type: reference
status: active
verified: 2026-04-29
---

# Alertmanager routing

Three replicas on `ops-tools`, gossip between them, configuration in `clusters/ops-tools/monitoring/alertmanager.yaml` rendered from a template so that receiver secrets come from the vault.

## Route tree (summary)

```yaml
route:
  receiver: chat-ops-warn
  group_by: [alertname, namespace]
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 24h
  routes:
    - matchers: [severity="page", team="platform"]
      receiver: pager-platform
      repeat_interval: 4h
    - matchers: [severity="page", team="ops"]
      receiver: pager-ops
      repeat_interval: 4h
    - matchers: [severity="page", team="mobile"]
      receiver: pager-mobile
      repeat_interval: 4h
    - matchers: [severity="warn", team="platform"]
      receiver: chat-platform-warn
    - matchers: [severity="warn", team="mobile"]
      receiver: chat-mobile-warn
    - matchers: [team="data"]
      receiver: chat-data
```

Anything without a `team` label falls to `chat-ops-warn`, and a weekly report lists such alerts so the label gets added. There are none at the moment.

## Receivers

- `pager-*`: webhook to the paging service, one escalation policy per team. The service calls the on-call phone, escalates to the secondary after 15 minutes without acknowledgement (see the platform on-call note for the commitment), then to the team lead. The on-call schedule is in the paging service and mirrored to the shared calendar.

- `chat-*-warn`: webhook to the team channel in the chat tool, one message per group with the summary lines and a link to the dashboard from the `dashboard` annotation. Resolved notifications are sent for `warn` (people asked for them) and not for `page` (the on-call closes the page by hand after checking).

- No e-mail receiver, see [[oncall-prefers-chat-alerts]].

## Grouping choices

`group_by: [alertname, namespace]` means 40 pods of the API crashing produce one notification, not 40. `group_wait: 30s` is the delay before the first notification of a new group, long enough to gather the related alerts, short enough for a page. `group_interval: 5m` is how often a changed group is re-sent.

`repeat_interval: 4h` on pages: an unacknowledged page re-fires every 4 hours. The paging service escalates far earlier, so this is a safety net for a paging service outage.

## Inhibition

Two rules, listed in [[alerting-rules-catalogue]]. Kept deliberately short.

## Silences

Created with `amtool silence add` from the maintenance runbooks, or from the Alertmanager UI through SSO. Every silence needs a comment with a ticket or a runbook name. A nightly script lists silences older than 24 h and posts them to the ops channel; a silence older than 7 days is expired by the script unless its comment starts with `long-lived:` and names a ticket.

## Testing a route

`amtool config routes test --config.file alertmanager.yaml severity=page team=mobile` prints the receiver. It runs in CI on every change to the file, for a fixed set of label combinations, and the expected receiver is asserted. This caught a typo (`team="plateform"`) once, before it reached production.

## What happens when Alertmanager is down

Prometheus keeps evaluating and retries sending. Three replicas across the three `ops-tools` nodes have not all been down at once. A dead-man's switch alert (`Watchdog`, always firing) is routed to the paging service's heartbeat endpoint; if the heartbeat stops for 10 minutes, the paging service pages ops on its own. That is the one alert that does not need Prometheus or Alertmanager to be alive to be noticed.
