---
name: prometheus-stack-layout
description: Metrics are scraped by a Prometheus agent per cluster and remote-written to a central Prometheus on ops-tools (15 s scrape, 30 d retention, 2.1 M active series), with recording rules per team and the exporters list
type: reference
status: active
verified: 2026-05-11
---

# Prometheus stack layout

## Topology

- **Agents** (Prometheus in agent mode, one per cluster, 2 replicas on `hf-main` with sharding by target hash) scrape everything in their cluster every 15 s and `remote_write` to the central server. Local retention is a 2 h WAL, enough to survive a central outage of that length without gaps.

- **Central Prometheus** on `ops-tools`, 1 replica (a second replica is on the list, see [[incident-2026-02-prometheus-oom]] for why it is not there yet), 30 days retention, 600 GB Longhorn volume, 24 GB memory request. Receives about 140 000 samples per second.

- **Alertmanager** 3 replicas on `ops-tools`, see [[alertmanager-routing-oncall]].

- **Grafana** on `ops-tools`, SSO, dashboards provisioned from Git, see [[dashboards-conventions]].

The central server evaluates all alerting and recording rules. The agents evaluate none (agent mode cannot).

## Discovery

`ServiceMonitor` and `PodMonitor` objects through the Prometheus operator, one per component in its Kustomize directory. A component without a monitor is not scraped, and the CI lint on `halden-infra` warns when a Deployment exposes a `metrics` port without a monitor.

## Exporters and what they cost

| Source | Active series | Notes |
|---|---|---|
| kube-state-metrics + cAdvisor | 620 000 | the bulk, dropped labels listed below |
| node exporter (16 nodes) | 90 000 | |
| halden-api (php-fpm, app metrics) | 310 000 | route label bounded to the route name, never the path |
| ingress-nginx | 180 000 | |
| PostgreSQL (operator exporter), see [[postgres-exporter-metrics]] | 40 000 | |
| RabbitMQ, Redis, PgBouncer | 35 000 | |
| Cilium and Hubble | 420 000 | Hubble metrics are the second largest, flow labels limited to namespace and verdict |
| Loki, ArgoCD, cert-manager, Longhorn, Velero, ESO | 160 000 | |
| Blackbox probes, see [[blackbox-probes-external]] | 2 000 | |
| Mobile app telemetry (pushed through a gateway) | 120 000 | |

Total 2.1 M active series (May 2026), up from 1.4 M a year earlier.

## Dropped at scrape

`metric_relabel_configs` on the agents drop: cAdvisor `container_*` for `pod=""` (pause containers), `container_network_*` per interface (only totals kept), `kube_pod_container_status_*_reason` cardinality by dropping `reason` on terminated states, and `apiserver_request_duration_seconds_bucket` for `verb=WATCH`. This removed 700 000 series in 2025. The list is in `clusters/hf-main/monitoring/agent-relabel.yaml` with a comment per rule.

## Recording rules

Per team, in `clusters/ops-tools/monitoring/rules/<team>.yaml`:

- `hf:api_request_rate5m`, `hf:api_error_ratio5m`, `hf:api_latency_p99_5m` by route (the SLO inputs, see [[slo-api-latency]])

- `hf:node_cpu_utilisation5m`, `hf:namespace_memory_requests_ratio`

- `hf:mobile_sync_error_ratio5m`

Dashboards query recording rules, not raw series, whenever the panel is on a home dashboard. Ad hoc panels can query raw.

## Federation and long-term storage

None. Thirty days is what we need for capacity planning and incident review. Monthly capacity numbers are exported by a script into a spreadsheet the team keeps, which is unglamorous and works.

## Access

Grafana through SSO for everyone in engineering. The Prometheus UI is reachable through a port-forward for ops only, since it has no auth of its own.
