---
name: longhorn-storage-lessons
description: Longhorn 1.7 with 3 replicas is fine for RabbitMQ, Redis and Loki WAL but was removed from under PostgreSQL after write latency of 4 to 9 ms and a rebuild storm during node replacement, replica rebuild throttled to 1 concurrent per node
type: feedback
status: active
verified: 2026-01-21
---

# Longhorn : ce qu'on a appris

## Où Longhorn reste

Volumes répliqués 3 fois (`numberOfReplicas: 3`, `dataLocality: best-effort`, `replicaAutoBalance: least-effort`), classe `longhorn-replicated`, pour :

- RabbitMQ (3 nœuds, 20 Go chacun)

- Redis avec persistance AOF (2 volumes de 8 Go)

- Le WAL de Loki et l'index local (voir les notes d'observabilité)

- Le registre miroir (volume de 400 Go, voir [[registry-harbor-mirror]])

- Quelques PVC de services de la plateforme data

Ces charges écrivent peu ou tolèrent la latence. Pour elles, Longhorn fait ce qu'on attend : un volume survit à la perte d'un nœud, la migration d'un pod vers un autre nœud prend 20 s.

## Où Longhorn a été retiré : PostgreSQL

En 2025, le primaire PostgreSQL tournait sur un volume Longhorn. Mesures en octobre 2025 avec `pg_test_fsync` et sous charge réelle :

- Latence d'écriture `fsync` : 4 à 9 ms sur Longhorn, 0,08 ms sur le NVMe local.

- `wal_write_time` par transaction : 12× plus élevé.

- Pendant le remplacement de `hf-wk-04` (voir [[runbook-node-drain-replace]]), Longhorn a reconstruit 6 réplicas en parallèle, saturant le réseau des deux nœuds concernés, et le p99 de l'API est passé à 2 s pendant 40 minutes.

Décision HF-1215 (novembre 2025) : PostgreSQL sur NVMe local (`local-path`), 2 nœuds dédiés, réplication gérée par l'opérateur, voir [[postgres-operator-cloudnative]]. La disponibilité vient de la réplication PostgreSQL, pas du stockage. Une panne d'un nœud base = bascule de l'opérateur en 30 s. Une panne des deux = restauration depuis la sauvegarde (voir [[backup-velero-schedule]] pour le calendrier), ce qu'on accepte pour deux nœuds dans deux racks distincts.

## Réglages qui ont réglé la tempête de reconstruction

- `concurrent-replica-rebuild-per-node-limit: 1` (défaut 5).

- `replica-replenishment-wait-interval: 600` : Longhorn attend 10 minutes avant de considérer un réplica comme perdu et d'en reconstruire un, ce qui couvre un redémarrage de nœud sans déclencher de reconstruction.

- `node-drain-policy: block-if-contains-last-replica`, pour qu'un drain ne puisse pas supprimer le dernier réplica sain d'un volume.

- Bande passante de reconstruction limitée par une `NetworkPolicy` avec annotation Cilium de limitation à 200 Mbit/s sur les pods `instance-manager`. Pas élégant, mais Longhorn n'a pas de limiteur de bande passante natif.

## Sauvegardes de volumes

Sauvegardes Longhorn (snapshots incrémentaux vers le magasin d'objets) toutes les 6 heures sur les volumes RabbitMQ et Redis, rétention 7 jours. Ce n'est pas la sauvegarde principale (Velero), c'est ce qui permet de récupérer une file RabbitMQ perdue sans tout restaurer.

## Ce qu'on surveille

- `longhorn_volume_robustness != 0` (dégradé ou en erreur) pendant plus de 15 minutes : `warn`.

- `longhorn_node_storage_usage_bytes / capacity > 0.8` : `warn`. Les nœuds généraux ont 1,9 To de NVMe dont 1 To réservé à Longhorn.

- Nombre de réplicas en reconstruction simultanément > 2 : `warn`, c'est le signe avant-coureur d'une tempête.

## Mise à jour

Longhorn se met à jour après RKE2, jamais en même temps, voir [[rke2-upgrade-1-31-to-1-32]]. Une mise à jour de Longhorn redémarre les `instance-manager` nœud par nœud et chaque volume passe brièvement en dégradé, c'est attendu et l'alerte est mise en silence pendant la fenêtre.
