---
name: alerting-rules-catalogue
description: Alert rules live in Git per team with severity page or warn, every page rule has runbook and summary annotations checked by a test, the catalogue lists the 14 rules that page and their thresholds as of spring 2026
type: reference
status: active
verified: 2026-05-25
---

# Alerting rules catalogue

Replaces [[alerting-rules-v1]]. Rules are in `clusters/ops-tools/monitoring/rules/<team>.yaml`, one `PrometheusRule` per team (`platform`, `ops`, `data`, `mobile`), loaded by the central Prometheus. Routing is in [[alertmanager-routing-oncall]].

## Conventions

- `labels.severity` is `page` or `warn`. Nothing else. `critical` and `info` were removed with v1 because nobody agreed what they meant.

- `labels.team` decides who receives it.

- `annotations.summary` is one line with the instance and the value (`API p99 latency is 4.2s on route api_loads_search`).

- `annotations.runbook` is a URL into the ops handbook, mandatory for `page`. A CI test (`promtool test rules` plus a `yq` check) fails when a `page` rule has no runbook.

- `for:` is never below 2 minutes on `page` rules, to survive a scrape hiccup. The exceptions are backup and certificate rules, which use long windows anyway.

- Every rule has a unit test in `rules/tests/<team>_test.yaml` with at least one firing and one non-firing case. `promtool test rules` runs in CI.

## Rules that page (spring 2026)

| Alert | Expression (short) | For | Team |
|---|---|---|---|
| `ApiErrorRateHigh` | `hf:api_error_ratio5m > 0.02` | 5m | platform |
| `ApiLatencyP99High` | `hf:api_latency_p99_5m > 3` | 10m | platform |
| `MessengerFailedQueueGrowing` | `increase(messenger_failed_messages_total[15m]) > 100` | 0m | platform |
| `OutboxRelayDown` | `outbox_pending > 0 and rate(outbox_delivered_total[5m]) == 0` | 5m | platform |
| `MobileSyncErrorRateHigh` | `hf:mobile_sync_error_ratio5m > 0.1` | 5m | mobile |
| `InvoicingRunFailed` | `hf_invoicing_run_status != 1` | 0m | platform |

### Ops rules that page

| Alert | Expression (short) | For | Team |
|---|---|---|---|
| `PostgresWalArchiveFailing` | `cnpg_pg_stat_archiver_failed_count increase > 0` | 10m | ops |
| `PostgresBackupTooOld` | `time() - cnpg_backup_last_success > 36h` | 0m | ops |
| `CertificateExpiringSoon` | `probe_ssl_earliest_cert_expiry - time() < 21d` | 1h | ops |
| `EtcdDbSizeHigh` | `etcd_mvcc_db_total_size_in_bytes > 0.8 × quota` | 5m | ops |

### Ops rules that page (continued)

| Alert | Expression (short) | For | Team |
|---|---|---|---|
| `VeleroBackupFailed` | `increase(velero_backup_failure_total[1h]) > 0` | 0m | ops |
| `RegistryDown` | `probe_success{job="registry"} == 0` | 5m | ops |
| `ArgoCDSyncRateHigh` | `increase(argocd_app_sync_total[10m]) > 20` | 0m | ops |
| `LokiIngesterDown` | `up{job="loki-write"} == 0` | 5m | ops |

Fourteen rules. The list grows by one or two a quarter and each addition is discussed in the ops and platform sync. A rule that has never fired in a year is reviewed for removal.

## Rules that warn (selection)

About 60. The ones that matter most: `PgBouncerClientsWaiting`, `KubeNodeMemoryPressure`, `LonghornVolumeDegraded`, `LokiStreamRateLimited`, `PartitionDefaultNotEmpty`, `CoreDNSServfailHigh`, `IngressUpstream5xxHigh`, `WafBlockedRequestsSpike`, `CrashFreeRateLow` (mobile), `WebBootErrorSpike` (this one was promoted to page after the Safari incident, so it is in the platform list as well).

## Silences and inhibition

- Planned maintenance silences are created from the runbook with a matcher on `node` or `namespace` and an end time. No open-ended silences; a script expires any silence older than 24 h and reports it.

- Inhibition: `KubeNodeNotReady` inhibits every `warn` from pods on that node. `ApiErrorRateHigh` inhibits `IngressUpstream5xxHigh` on the API ingress. That is the whole inhibition config; more than that and nobody understands why an alert did not fire.

## Adding a rule

Copy a neighbour, write the test first (the firing case), set `severity: warn` for two weeks in production, look at how often it fires, then promote to `page` if the on-call agrees. The SLO rules in [[slo-api-latency]] followed that path.

## A complete rule with its test

`OutboxRelayDown` as it is in `rules/platform.yaml`, because it is the one people copy:

```yaml
- alert: OutboxRelayDown
  expr: |
    max(hf_outbox_pending_rows{namespace="platform-prod"}) > 0
    and rate(hf_outbox_delivered_total{namespace="platform-prod"}[5m]) == 0
  for: 5m
  labels:
    severity: page
    team: platform
  annotations:
    summary: "Outbox relay has delivered nothing for 5m with {{ $value }} rows pending"
    runbook: https://handbook.hf.internal/runbooks/outbox-relay
    dashboard: https://grafana.hf.internal/d/hf-api-overview
```

And its test in `rules/tests/platform_test.yaml`:

```yaml
- interval: 1m
  input_series:
    - series: 'hf_outbox_pending_rows{namespace="platform-prod"}'
      values: '0 0 12 40 90 120 150 180 200 220'
    - series: 'hf_outbox_delivered_total{namespace="platform-prod"}'
      values: '100 110 120 120 120 120 120 120 120 120'
  alert_rule_test:
    - eval_time: 9m
      alertname: OutboxRelayDown
      exp_alerts:
        - exp_labels: { severity: page, team: platform, namespace: platform-prod }
          exp_annotations:
            summary: "Outbox relay has delivered nothing for 5m with 220 rows pending"
    - eval_time: 4m
      alertname: OutboxRelayDown
      exp_alerts: []
```

The non-firing case at 4 minutes is what catches an accidental `for: 0m`, and the `summary` assertion is what catches a broken template (a `$value` outside the quotes once produced a YAML parse error at Prometheus reload time, which is a silent failure until the next rule load).

`{{ $value }}` in the summary is rounded by `humanize` only for float metrics; row counts are printed raw on purpose. A summary that says `220` is more useful at 3 in the morning than `220.0`.

## When a rule fires and should not have

The process is a one-line change in the same MR as the fix: raise the threshold or the `for`, add a test case that reproduces the false positive as a non-firing case, and note the date in a comment above the rule. Four rules carry such a comment today. `ApiLatencyP99High` has the longest one, from the first month of the SLO work, when the `for` went from 5 to 10 minutes after two pages caused by a 6 minute garbage collection pause in the ingress controller that had nothing to do with the API.
