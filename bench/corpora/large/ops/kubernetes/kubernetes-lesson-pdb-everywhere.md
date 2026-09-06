---
name: kubernetes-lesson-pdb-everywhere
description: Every Deployment with 2 or more replicas needs a PodDisruptionBudget with minAvailable strictly below replicas, enforced by admission and a lint, because a drain without one takes both replicas and a drain with a wrong one hangs forever
type: feedback
status: active
verified: 2026-04-02
---

# PDB partout, et correct

## La règle

Tout Deployment ou StatefulSet de plus d'un réplica a un `PodDisruptionBudget` avec `minAvailable` (ou `maxUnavailable`) tel que **au moins un pod peut être évincé**. Concrètement : `minAvailable < replicas`, ou `maxUnavailable >= 1`. Un pod seul n'a pas de PDB (une PDB avec `minAvailable: 1` sur un réplica bloque tout drain).

Deux mécanismes l'imposent :

- Une politique d'admission Kyverno (`require-pdb`) en `enforce` sur `platform-prod` et `ops` refuse un Deployment de 2 réplicas ou plus sans PDB portant le même sélecteur.

- Un lint dans le CI de `halden-infra` (`scripts/lint-pdb.sh`, 40 lignes de `yq`) vérifie que `minAvailable < replicas` pour chaque paire, et que le sélecteur de la PDB correspond bien aux labels du template. C'est le lint ajouté l'après-midi du blocage de drain sur `live-gw` pendant [[rke2-upgrade-1-31-to-1-32]].

## Pourquoi les deux bords

**Sans PDB** : un drain évince tous les pods d'un nœud en même temps. Avec deux réplicas de `live-gw` sur deux nœuds, ce n'est pas un problème. Avec deux réplicas sur le même nœud (ce que `ScheduleAnyway` autorise quand l'autre rack est plein), un drain les prend tous les deux et le service tombe pendant 10 s. C'est arrivé au relais outbox... non, à `pgbouncer-api` en 2025, et l'API a vu 8 s d'erreurs de connexion.

**Avec une PDB fausse** : `minAvailable: 2` sur un Deployment de 2 réplicas, et le drain attend indéfiniment. Le `--timeout 600s` du runbook ([[runbook-node-drain-replace]]) fait échouer le drain au lieu de le laisser pendre, mais on a perdu 10 minutes à comprendre.

## Ce qu'on met

| Composant | replicas | PDB |
|---|---|---|
| `halden-api` | 12 à 60 (HPA) | `maxUnavailable: 20%` |
| `halden-api-worker-*` | 1 à 12 (KEDA) | `maxUnavailable: 1` |
| `live-gw` | 2 | `minAvailable: 1` |
| `pgbouncer-api` | 2 | `minAvailable: 1` |
| `halden-web` | 3 | `minAvailable: 2` |
| CoreDNS | 4 | `minAvailable: 2` |
| RabbitMQ | 3 | `maxUnavailable: 1` |

Pourcentage sur ce qui est scalé automatiquement, sinon la PDB devient fausse quand l'HPA descend au minimum. `maxUnavailable: 1` sur les StatefulSets qui ont un quorum.

## Comment l'appliquer ailleurs

- Écrire la PDB dans le même fichier que le Deployment, juste en dessous. Une PDB dans un autre fichier est oubliée à la suppression du Deployment et bloque le drain pour un sélecteur qui ne correspond plus à rien (oui, ça bloque aussi : Kubernetes compte zéro pods disponibles sur zéro voulus... en fait non, une PDB sans pods n'empêche rien, mais elle pollue `kubectl get pdb`).

- Avant tout drain, `kubectl get pdb -A` et regarder la colonne `ALLOWED DISRUPTIONS`. Un zéro est soit un problème en cours (pod pas prêt), soit une PDB fausse.

- La PDB ne protège pas contre une panne de nœud, seulement contre les évictions volontaires. La protection contre la panne, c'est la répartition par zone, voir [[node-pools-and-taints]].
