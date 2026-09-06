---
name: node-pools-and-taints
description: Node labels and taints on hf-main, workload=database and workload=ocr taints with matching tolerations, ingress=true label for the ingress DaemonSet, rack zone labels for topology spread, and the affinity rules the platform manifests must carry
type: reference
status: active
verified: 2026-03-12
---

# Pools de nœuds, labels et taints

Voir [[rke2-cluster-layout]] pour le matériel. Ici : ce qu'un manifeste doit déclarer pour atterrir au bon endroit.

## Labels

| Label | Valeurs | Posé par |
|---|---|---|
| `node-role.kubernetes.io/control-plane` | `true` | RKE2 |
| `hf.example/pool` | `general`, `database`, `ocr`, `ingress` | playbook de provisionnement |
| `ingress` | `true` sur `hf-wk-01`, `hf-wk-02` | playbook |
| `topology.kubernetes.io/zone` | `rack-a`, `rack-b` | playbook |
| `nvidia.com/gpu.present` | `true` sur `hf-ocr-*` | device plugin |

## Taints

- Control-plane : `node-role.kubernetes.io/control-plane:NoSchedule`. Tolérée par les DaemonSets du namespace `ops` et par CoreDNS. Rien d'applicatif ne tourne dessus.

- `hf-db-*` : `workload=database:NoSchedule`. Seuls les pods PostgreSQL de l'opérateur et l'exporteur la tolèrent. Un pod applicatif qui la tolère est refusé en revue.

- `hf-ocr-*` : `workload=ocr:NoSchedule`. Tolérée par les workers OCR uniquement, voir [[ocr-workers-node-pool]].

- Les nœuds d'ingress ne sont **pas** taintés : ils portent aussi des charges générales, le contrôleur d'ingress en `hostNetwork` n'occupe que les ports 80 et 443.

## Ce qu'un Deployment de la plateforme doit déclarer

```yaml
topologySpreadConstraints:
  - maxSkew: 1
    topologyKey: topology.kubernetes.io/zone
    whenUnsatisfiable: ScheduleAnyway
    labelSelector: { matchLabels: { app: halden-api } }
  - maxSkew: 1
    topologyKey: kubernetes.io/hostname
    whenUnsatisfiable: ScheduleAnyway
    labelSelector: { matchLabels: { app: halden-api } }
```

`ScheduleAnyway` et pas `DoNotSchedule` : on préfère un pod mal placé à un pod non planifié quand un rack est en maintenance. La règle est vérifiée par une politique d'admission (Kyverno, mode `audit` en staging, `enforce` en prod) qui refuse un Deployment de plus d'un réplica sans contrainte de répartition par zone. Voir [[kubernetes-lesson-pdb-everywhere]] pour la politique jumelle sur les PDB.

Pas d'`affinity` par nom de nœud dans les manifestes applicatifs. Le seul cas légitime est le pinning des pods PostgreSQL par l'opérateur, et il le fait lui-même.

## Réservations sur les nœuds

Kubelet configuré avec `system-reserved: cpu=1,memory=2Gi` et `kube-reserved: cpu=1,memory=2Gi` sur les nœuds généraux, et `eviction-hard: memory.available<1Gi,nodefs.available<10%`. Les pods sans `requests` mémoire sont refusés par la politique d'admission, voir [[hpa-and-resource-requests]].

## Ajouter un nœud

1. Provisionnement par le playbook (`make node NAME=hf-wk-08 POOL=general ZONE=rack-b`), qui installe RKE2 agent, pose les labels et taints, et enregistre le nœud.

2. Vérifier `kubectl get node hf-wk-08 -o yaml | grep -A5 labels`.

3. Longhorn le découvre seul et commence à y placer des réplicas, ce qui déclenche des reconstructions : le limiteur de [[longhorn-storage-lessons]] les rend supportables.

4. Retirer le cordon posé par le playbook une fois que Cilium et les exporteurs sont `Ready`.

Un nœud ajouté sans passer par le playbook n'a pas les labels de zone, et tous les `topologySpreadConstraints` deviennent faux. C'est arrivé une fois.
