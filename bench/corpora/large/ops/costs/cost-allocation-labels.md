---
name: cost-allocation-labels
description: Deux étiquettes Kubernetes obligatoires, hf.cost/team et hf.cost/service, sur tout Deployment, StatefulSet, CronJob et PVC, valeurs dans une liste fermée, refusées par la CI si absentes depuis mars 2026, la requête Prometheus qui fait la répartition, et le cas des ressources partagées
type: reference
status: active
verified: 2026-04-08
---

## Les deux étiquettes

| Étiquette | Valeurs | Sert à |
|---|---|---|
| `hf.cost/team` | `platform`, `data`, `product`, `ml`, `ops`, `billing`, `dispatch` | savoir qui reçoit la ligne à la revue mensuelle |
| `hf.cost/service` | `api`, `auth`, `notifications`, `dispatch-web`, `driver-backend`, `warehouse`, `torrent`, `storage`, `ml`, `ocr`, `maps`, `onboarding`, `ci`, `staging`, `monitoring`, `control-plane` | la colonne de [[per-service-cost-table-q2-2026]] |

Les valeurs sont dans `finops/labels.yaml` et nulle part ailleurs. Une valeur qui n'y est pas est refusée par la CI (un test sur les manifestes rendus par Kustomize, depuis mars 2026, HF-4715) et par une politique d'admission sur le cluster (`warn` seulement, pour ne pas bloquer un correctif urgent ; la CI est la vraie barrière). Un service nouveau commence par une MR sur `labels.yaml`, ce qui oblige à dire à quelle équipe il appartient avant d'exister.

Les deux étiquettes sont sur le contrôleur (Deployment, StatefulSet, CronJob) **et** propagées au modèle de pod, parce que les métriques de conteneur portent les étiquettes du pod, pas celles du contrôleur. Les PVC portent les deux aussi, pour le stockage par blocs (Longhorn). Les buckets du stockage objet ne sont pas des objets Kubernetes ; leur propriétaire est dans l'inventaire du stockage, et le coût du stockage objet est réparti par bucket, pas par étiquette.

## La requête

La part de calcul d'un service sur le mois, en cœurs demandés (pas utilisés, voir [[rightsizing-2025-q4-requests-limits]] pour l'argument) :

```
avg_over_time(
  sum by (label_hf_cost_service) (
    kube_pod_container_resource_requests{resource="cpu"}
    * on (namespace, pod) group_left(label_hf_cost_service)
      kube_pod_labels
  )[30d:1h]
)
```

Même chose pour `resource="memory"`, et `kubelet_volume_stats_capacity_bytes` joint aux étiquettes des PVC pour le stockage par blocs. Le coût d'un cœur-mois et d'un Go-mois vient de la ligne d'amortissement et de colocation divisée par la capacité totale des nœuds de travail : 21 EUR par cœur demandé et par mois, 2,60 EUR par Go de mémoire demandé et par mois en 2026. Un pod qui demande 500 m et 1 Gi coûte donc 13 EUR par mois dans la table, quoi qu'il fasse.

`finops-report` exécute ces requêtes le premier du mois et écrit le résultat dans `finops/allocations/2026-06.yaml`, qui est relu avec la table.

## Les ressources partagées

- **Le plan de contrôle, l'ingress, ArgoCD, cert-manager, la surveillance** : étiquetés `control-plane` ou `monitoring`, équipe `ops`, et montrés en `shared` dans la table. On ne les répartit pas au prorata : une ligne `shared` visible vaut mieux qu'un supplément invisible sur chaque service.

- **PostgreSQL principal** : un seul cluster pour l'API, l'auth et la facturation. Étiqueté `api`, avec une répartition manuelle dans `costs.yaml` (`api` 70 %, `auth` 10 %, `billing` 20 %, d'après la taille des schémas et le nombre de requêtes du mois d'octobre 2025, revue annuellement). C'est la seule répartition manuelle du calcul.

- **Les nœuds GPU** : étiquetés `ocr`, et la part utilisée par `ml-infer` en dehors des heures d'OCR est notée dans [[gpu-vs-cpu-inference-cost]] plutôt que répartie.

- **Les nœuds à la demande du pool** : étiquetés par l'équipe qui les a pris, le mois où elle les a pris ([[reserved-capacity-decision-2026-01]]).

## Couverture

| Mois | Part des cœurs demandés sans étiquette valide |
|---|---|
| 2025-10 | 11 % |
| 2025-12 | 6 % |
| 2026-03 (CI bloquante) | 2,5 % |
| 2026-06 | 2 % |

Les 2 % restants sont des pods d'opérateurs installés par des charts tiers qui ne propagent pas nos étiquettes ; ils sont dans `shared` et la liste des dix plus gros est dans le rapport mensuel.

## Ce qu'on ne fait pas

- Pas d'étiquette `hf.cost/environment` : l'environnement est le cluster ou l'espace de noms, déjà connu. Staging a son propre nœud et sa propre ligne ([[staging-environment-cost-cut]]).

- Pas d'étiquette de « centre de coût » comptable : la correspondance équipe → centre de coût est une table de trois lignes chez la finance, pas une étiquette à maintenir sur 400 manifestes.

- Pas de refacturation réelle. La table est une répartition pour décider, personne ne reçoit de facture interne ([[finops-monthly-review-feedback]] sur pourquoi c'est important).
