---
name: rke2-cluster-layout
description: One RKE2 cluster (v1.32 as of May 2026) with 3 control-plane nodes and 11 workers across two racks in one datacentre, namespaces platform-prod, platform-staging, data, ops, with a second cluster ops-tools for ArgoCD, monitoring and the registry
type: reference
status: active
verified: 2026-05-28
---

# Cluster layout

## The two clusters

**`hf-main`** runs the workloads. RKE2, Kubernetes v1.32.4 since the upgrade described in [[rke2-upgrade-1-31-to-1-32]]. Bare metal in one datacentre near Lyon, two racks, our own hardware on a 4 year cycle.

**`ops-tools`** is small (3 nodes, control-plane and worker combined, tainted so only ops workloads schedule) and runs what must survive `hf-main` being broken: ArgoCD (see [[argocd-app-of-apps]]), the Prometheus and Loki stack, the container registry mirror ([[registry-harbor-mirror]]), the secrets operator's control plane, and the backup controller. When we rebuilt `hf-main` in 2025 after the etcd disaster rehearsal, `ops-tools` is what let us redeploy everything from Git in 40 minutes.

## `hf-main` nodes

| Role | Count | Hardware | Notes |
|---|---|---|---|
| control-plane | 3 | 8 cores, 32 GB, 2× NVMe 480 GB | etcd on its own NVMe, see [[incident-2026-04-etcd-disk-full]] |
| worker general | 7 | 32 cores, 128 GB, 2× NVMe 1.9 TB | API, workers, web, live-gw |
| worker database | 2 | 32 cores, 256 GB, 4× NVMe 3.8 TB | PostgreSQL primary and replica, tainted `workload=database:NoSchedule` |
| worker ocr | 2 | 16 cores, 64 GB, 1 GPU | document OCR workers, see [[ocr-workers-node-pool]] |

Details of taints and labels in [[node-pools-and-taints]]. Nodes are named `hf-cp-01..03`, `hf-wk-01..07`, `hf-db-01..02`, `hf-ocr-01..02`. Rack is a label `topology.kubernetes.io/zone: rack-a|rack-b` and every stateful workload has a topology spread constraint on it.

## Networking

- CNI: Cilium in kube-proxy replacement mode, VXLAN overlay, network policies enforced (see [[network-policies-baseline]]).

- Ingress: ingress-nginx behind two nodes with a VIP managed by kube-vip, see [[ingress-nginx-config]]. The former Traefik setup is in [[ingress-traefik-legacy]].

- Pod CIDR `10.42.0.0/16`, service CIDR `10.43.0.0/16`, node network `10.20.0.0/24`. Nothing routes from the pod network to the office network; access is through the VPN to a bastion.

- DNS: CoreDNS with the tuning in [[cluster-dns-coredns-tuning]].

## Storage

Longhorn for replicated block volumes (3 replicas, used by RabbitMQ, Redis persistence, Loki's WAL). PostgreSQL does **not** use Longhorn: it runs on local NVMe through the `local-path` provisioner on the database nodes, with replication handled by the operator, see [[postgres-operator-cloudnative]] and [[longhorn-storage-lessons]]. Object storage is a separate appliance in the same racks, S3 API, 60 TB usable, used for documents, backups and the tile server's data.

## Namespaces on `hf-main`

- `platform-prod`, `platform-staging`: the application, one namespace per environment, same manifests with Kustomize overlays. Staging and prod on the same cluster was a deliberate choice: one cluster to operate, and the isolation is namespaces plus network policies plus resource quotas (staging is capped at 20 % of the general workers' capacity). A staging bug cannot take prod down through the cluster, but it can through a shared ingress misconfiguration, which is why ingress changes are reviewed by ops.

- `data`: the data platform (streaming ingestion, the warehouse loader, the tile server).

- `ops`: node-level agents (Longhorn, monitoring exporters, the secrets operator's agent, cert-manager).

- `kube-system`: RKE2's own components and Cilium.

## Capacity (May 2026)

General workers: 224 cores, 896 GB allocatable. Requested at steady state: 140 cores, 520 GB. Peak usage 7:30 on Mondays: 165 cores. The rule of thumb is to keep 25 % headroom on requests so that losing one general worker (32 cores) still fits. We are at the limit and the eighth general worker is ordered.

## Access

`kubectl` through the VPN and a bastion with OIDC against the company identity provider. Groups: `ops-admin` (cluster-admin), `platform-dev` (edit in `platform-staging`, view in `platform-prod`), `platform-oncall` (edit in `platform-prod` during on-call, granted by a scheduled job that syncs the calendar). No long-lived kubeconfig for humans. Service accounts for CI have namespace-scoped roles.

## Things to know before touching it

- The control-plane nodes also run the Longhorn managers and CoreDNS. They are not tainted `NoSchedule` for those, only for application pods (`node-role.kubernetes.io/control-plane:NoSchedule` with tolerations on the ops DaemonSets).

- Draining a database node is a planned operation, see [[runbook-node-drain-replace]], and takes 15 minutes because of the PostgreSQL switchover.

- The two OCR nodes are the only ones with a GPU and the only ones with a different kernel. Upgrade them last.

- Everything in `ops` and `kube-system` is managed by ArgoCD from `halden-infra`. A change made by `kubectl edit` is reverted within 3 minutes by self-heal, which is the intended behaviour and surprises every new person once.

## Hardware lifecycle, spares and out-of-band access

Servers are on a 4 year replacement cycle, bought in batches so that a pool is homogeneous. The general workers were bought in two batches (4 in 2023, 3 in 2024), which is why `hf-wk-05` to `-07` have a slightly newer CPU generation and about 8 % more single-thread performance. The scheduler does not know, and the API's latency histogram by node shows the difference if you look for it. The eighth worker on order is from the 2024 batch's successor and will start a third generation in the pool.

Spares kept in the racks: two NVMe drives of each size in use, one power supply per chassis model, and one complete general worker chassis (`hf-wk-spare`, powered off, not registered in the cluster). The spare chassis has been used once, for the `hf-wk-04` replacement in October 2025, and was replaced by a new purchase the following month. A database node failure would be handled by re-provisioning onto the spare chassis with the 4 NVMe drives moved over, which is rehearsed on paper only, and that is a known gap.

Firmware: BIOS and NIC firmware are updated once a year during the January maintenance window, one node at a time through the drain runbook. The NVMe firmware is not updated unless the vendor's advisory names our model, which happened once in 2025 for a power-loss data corruption bug, and that update was done on the database nodes first, on a Saturday, with a full backup taken before.

Out-of-band management (IPMI) is on a separate VLAN reachable only from the bastion, with per-node credentials in the vault and a script that rotates them quarterly. The IPMI web interfaces are old and ugly and the only way to see a kernel panic message, which is how the January 2026 `hf-db-01` panic was diagnosed (a NIC driver bug, fixed by a kernel update on both database nodes the following week).

Power: each rack has two feeds from two distribution units. Every chassis has two power supplies, one per feed. A feed loss was tested in 2025 by pulling the breaker on rack B's feed A: nothing rebooted. Cooling is the datacentre's, and the only temperature alert we have is the node exporter's `node_hwmon_temp_celsius` above 80 degrees on any CPU, `warn`, which fired once during a cooling maintenance and was correct.

The physical inventory (serials, rack positions, purchase dates, warranty end) is a YAML file in `halden-infra/inventory/hardware.yaml`, and the provisioning playbook reads the node's serial from `dmidecode` and refuses to provision a node whose serial is not in it. This caught a mislabelled chassis once.
