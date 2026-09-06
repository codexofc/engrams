---
name: runbook-node-drain-replace
description: Runbook to drain, reboot or replace a node of hf-main, with the extra steps for ingress nodes (VIP), database nodes (PostgreSQL switchover) and OCR nodes (GPU module), and the checks before uncordon
type: reference
status: active
verified: 2026-05-09
---

# Runbook: drain, reboot or replace a node

Applies to `hf-main`. Written after the 2025 node replacements and revised after the [[rke2-upgrade-1-31-to-1-32]] upgrade.

## Before anything

1. Announce in the ops channel with the node name and expected duration.

2. Check `kubectl get pdb -A` for any PDB with `ALLOWED DISRUPTIONS 0`. A drain will hang on it. Fix the PDB or wait, see [[kubernetes-lesson-pdb-everywhere]].

3. Check Longhorn: `kubectl -n longhorn-system get volumes.longhorn.io | grep -v healthy` must be empty. Draining a node while a volume is degraded can remove the last healthy replica (the drain policy blocks it, but then the drain hangs).

4. Silence the alerts for the node in the alert manager for the expected duration plus 30 minutes.

## General worker (`hf-wk-*`)

```
kubectl cordon hf-wk-05
kubectl drain hf-wk-05 --ignore-daemonsets --delete-emptydir-data --timeout 600s
```

Expected 3 to 6 minutes. The API pods reschedule elsewhere (the HPA may add pods during the drain, that is fine). If the drain hangs past 5 minutes, `kubectl get pods -A --field-selector spec.nodeName=hf-wk-05` shows what is left, usually a pod with a PDB at zero.

Then reboot, or for a replacement: power off, swap, re-provision with the playbook under the same name (the playbook removes the old node object first).

## Ingress node (`hf-wk-01`, `hf-wk-02`)

Same as above, plus: check which node holds the VIP (`ip addr show | grep 10.20.0.50` on both) and drain the other one first if possible. When the VIP holder is drained, kube-vip moves the VIP in about 3 s and the live gateway's clients reconnect. Do it outside 07:00 to 09:30.

## Database node (`hf-db-*`)

1. Check which instance is primary: `kubectl cnpg status hf-postgres -n platform-prod`.

2. If the node to drain holds the primary: `kubectl cnpg promote hf-postgres <replica-instance> -n platform-prod`. Wait for the status to show the new primary and the old one as replica in streaming, about 30 s. The API sees a connection error burst for that duration and retries.

3. Check replication lag is under 1 s.

4. `kubectl drain hf-db-01 --ignore-daemonsets --delete-emptydir-data --timeout 900s`. The PostgreSQL pod on that node is evicted; with `local-path` storage it can only come back on the same node, so it stays `Pending` until uncordon. That is expected. For that duration the cluster runs on one instance, see [[postgres-operator-cloudnative]].

5. Reboot or replace. **A replacement of a database node means the local NVMe data is gone**: after re-provisioning, the operator rebuilds the replica from the primary with `pg_basebackup`, 1.2 TB, about 50 minutes. Do not replace both nodes in the same week.

6. Uncordon, wait for the instance to be `Ready` and streaming, lag under 1 s.

7. Optionally switch the primary back to its usual node. We do not bother; the two nodes are identical.

## OCR node (`hf-ocr-*`)

Same as general, plus after reboot: `nvidia-smi` on the node must list the GPU before uncordon, and the device plugin pod must be `Running`. If the kernel changed, rebuild the module (DKMS) first. See [[ocr-workers-node-pool]].

## Control-plane node (`hf-cp-*`)

Only one at a time. `etcdctl endpoint health` on the other two before starting. After reboot, `etcdctl endpoint status` must show three members and the same `RAFT INDEX` within a few units. A control-plane node replacement is a different runbook (etcd member removal and re-add), in the RKE2 section of the infra README.

## Before uncordon (every type)

- Node `Ready`, Cilium agent `Running` on it, `cilium status` clean.

- Longhorn node schedulable (`kubectl -n longhorn-system get nodes.longhorn.io hf-wk-05`).

- Node exporter and log agent pods `Running`.

- Kernel and RKE2 versions match the others (`kubectl get nodes -o wide`).

```
kubectl uncordon hf-wk-05
```

Then unsilence the alerts and close the announcement with the actual duration.

## Common mistakes

- Draining without `--delete-emptydir-data` and wondering why it hangs on the Prometheus agent.

- Forgetting the Longhorn health check and hitting the drain policy block.

- Replacing a node under a different name and leaving the old node object, which keeps a phantom in every topology spread calculation. The playbook handles it when the name is reused.

## Timings observed

Kept here so the announcement in the ops channel can carry a realistic duration.

| Operation | Node type | Date | Drain | Total |
|---|---|---|---|---|
| Reboot for kernel update | general | 2026-01-27 | 4 min | 12 min |
| Replacement (chassis swap) | general (`hf-wk-04`) | 2025-10-14 | 5 min | 2 h 10 (Longhorn rebuild storm, see the storage note) |
| Reboot for RKE2 upgrade | control-plane | 2026-03-30 | n/a | 12 min each |
| Reboot for RKE2 upgrade | database (with switchover) | 2026-03-30 | 3 min | 25 min for both |
| Reboot after kernel panic | database (`hf-db-01`) | 2026-01-09 | unplanned | 40 s failover, 55 min until replica caught up |
| Reboot for GPU module rebuild | ocr | 2026-03-30 | 2 min | 40 min |
| Ingress node reboot | ingress | 2025-11-18 | 4 min | 10 min, VIP moved in 3 s |

The replacement of a general worker is the only operation that took hours, and it was the Longhorn rebuild, not the swap. With the current rebuild limits it is estimated at 45 minutes.

## If the replaced node misbehaves

A re-provisioned node that joins the cluster but behaves oddly (pods scheduled on it crash-loop with network errors, Longhorn reports it unschedulable, metrics missing) is cordoned immediately, not debugged in place. The checks, in order: `cilium status` on the node (the agent must show `KVStore: Ok` and the right number of endpoints), `journalctl -u rke2-agent` for certificate or registration errors, and the `registries.yaml` content (a node provisioned with a stale playbook once came up without the mirror config and could not pull anything, which looked like a network problem).

If the node cannot be made healthy in 30 minutes, it is removed (`kubectl delete node`, then the playbook's `make node-remove` which also cleans the Longhorn node object and the etcd member if it was a control-plane node) and re-provisioned from scratch. A second failed provisioning means the hardware goes back to the vendor and the spare chassis is used. This has not happened yet.

Every operation on a node ends with a line in `halden-infra/inventory/maintenance-log.md`: date, node, what, duration, surprises. That file is where the timings table above comes from, and reading it before an operation is part of the announcement step.
