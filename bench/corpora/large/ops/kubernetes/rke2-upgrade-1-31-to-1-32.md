---
name: rke2-upgrade-1-31-to-1-32
description: RKE2 upgrade from v1.31.6 to v1.32.4 done 2026-03-30 with system-upgrade-controller, control plane first then workers one at a time, 3 h 10 total, blocked twice by a PDB and by the OCR nodes' kernel module
type: project
status: active
verified: 2026-04-08
---

# RKE2 v1.31 → v1.32 (HF-INFRA-412)

Done on Monday 2026-03-30 from 09:30 to 12:40, window announced a week ahead. No user-visible impact.

## Method

`system-upgrade-controller` with two `Plan` objects in `halden-infra/clusters/hf-main/upgrade/`:

1. `rke2-server`: control-plane nodes, `concurrency: 1`, cordon before, the plan waits for the node to be `Ready` and for etcd to report a healthy member before moving on. 3 nodes, about 12 minutes each.

2. `rke2-agent`: workers, `concurrency: 1`, `drain` with `--ignore-daemonsets --delete-emptydir-data --timeout 600s`, node selector excluding `hf-db-*` and `hf-ocr-*` which are done separately (below).

Both plans are applied by ArgoCD with auto-sync off; the upgrade is started by changing the `version` field in a MR and syncing by hand. Rolling back RKE2 is not supported, so the plan is tested first on a throwaway 3-node cluster built from the same playbooks (took a morning the week before).

## Pre-checks that mattered

- `kubectl get --raw /metrics | grep apiserver_requested_deprecated_apis` was empty. Nothing to migrate.

- Cilium 1.16 supports 1.32, Longhorn 1.7 supports 1.32, the CloudNativePG operator version we run supports 1.32. All checked against their compatibility tables and written in the MR.

- Every Deployment with more than one replica has a PDB, see [[kubernetes-lesson-pdb-everywhere]]. This is what made the worker drains safe.

- Velero backup of the whole cluster taken at 09:00, restore tested on the throwaway cluster the week before.

## What blocked

**Drain of `hf-wk-03` stuck for 10 minutes.** The `live-gw` Deployment had 2 replicas and a PDB with `minAvailable: 2` (a typo, should have been 1). The drain could not evict either pod. Fixed the PDB, drain continued. The PDB lint that checks `minAvailable < replicas` was added the same afternoon.

**OCR nodes.** The GPU driver kernel module was built against the previous kernel and the RKE2 package upgrade pulled a kernel update on those two nodes (they are on a different distribution channel from the general workers, a mistake from 2025). After reboot the OCR pods failed to start. Rebuilt the module with the vendor's DKMS package, 40 minutes lost. Those two nodes are now pinned to the same kernel channel as the rest and the OCR node pool has a pre-upgrade check for the module, see [[ocr-workers-node-pool]].

**Database nodes.** Done by hand, not by the plan: switchover of PostgreSQL to the replica (operator command, 30 s), drain and upgrade the former primary, wait for it to catch up, switch back. 25 minutes for both. See [[runbook-node-drain-replace]] for the same steps.

## After

- API p99 unchanged, no pod restarts outside the drains.

- The `etcd-arg` override added during this work is the one that caused [[incident-2026-04-etcd-disk-full]] two weeks later. The lesson is in that note.

- Cilium and Longhorn were upgraded on the following two Mondays, one component per week.

## Next

v1.33 in Q3 2026, same method. The throwaway cluster rehearsal is not optional and is now in the runbook as step 1.
