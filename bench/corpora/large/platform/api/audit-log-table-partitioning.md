---
name: audit-log-table-partitioning
description: load_events and audit_log are range-partitioned by month since HF-1380 (Jan 2026), partitions created 3 months ahead by a cron, detached and dropped after 24 months, done in three deploys
type: project
status: active
verified: 2026-03-27
---

# Partitionnement de `load_events` et `audit_log`

## Pourquoi

En décembre 2025, `load_events` faisait 41 M de lignes et 19 Go avec ses index, `audit_log` 28 M et 14 Go. La purge des lignes de plus de 24 mois par `DELETE` prenait 3 h et générait autant de bloat qu'elle en libérait. L'autovacuum n'arrivait plus à suivre sur ces deux tables. Et le dump nocturne passait 40 % de son temps dessus.

## Ce qui a été fait (HF-1380)

Partitionnement par plage sur `occurred_at` (respectivement `created_at`), une partition par mois, nommée `load_events_2026_03`.

En trois déploiements, à cause de [[incident-2025-11-migration-lock-loads]] :

1. **Déploiement 1** : création de `load_events_new` partitionnée, avec les mêmes index déclarés sur le parent. Trigger sur `load_events` qui duplique chaque `INSERT` vers la nouvelle table. Aucun verrou long.
2. **Entre les deux** : `app:backfill:load-events-partitions` copie l'historique par tranches de 100 000 lignes, mois par mois, en commençant par les plus anciens. 6 h en tout, en journée, à 15 % d'IO en plus, sans que personne ne le remarque. Vérification par `count(*)` par mois des deux côtés.
3. **Déploiement 2** : dans une transaction courte, `ALTER TABLE load_events RENAME TO load_events_old; ALTER TABLE load_events_new RENAME TO load_events;` puis suppression du trigger. Verrou de 40 ms.
4. **Déploiement 3** (une semaine plus tard, après avoir vérifié que rien ne manquait) : `DROP TABLE load_events_old`. 19 Go libérés d'un coup.

## Clé de partition et clé primaire

La clé primaire doit contenir la colonne de partitionnement, donc `PRIMARY KEY (id, occurred_at)`. Doctrine ne sait pas mapper une clé composite dont une moitié est un timestamp sans que les relations en souffrent. Solution : l'entité `LoadEvent` déclare `id` comme seul identifiant, et `schema:validate` est apaisé par une entrée dans `config/doctrine/schema_filter.yaml` qui exclut ces deux tables de la validation. Le schéma des deux tables est maintenu dans des migrations écrites à la main. Ce n'est pas élégant, c'est assumé.

Conséquence : pas de clé étrangère depuis d'autres tables vers `load_events`. Il n'y en avait aucune.

## Gestion des partitions

`bin/console app:partitions:maintain` en CronJob le 1er de chaque mois à 03 h :

- crée les partitions des 3 mois à venir si elles n'existent pas (`CREATE TABLE ... PARTITION OF ... FOR VALUES FROM (...) TO (...)`) ;
- détache (`DETACH PARTITION ... CONCURRENTLY`) puis supprime les partitions de plus de 24 mois ;
- lance `ANALYZE` sur le parent.

Une partition par défaut (`load_events_default`) existe et une alerte part si elle contient plus de 0 ligne (`PartitionDefaultNotEmpty`), parce que ça voudrait dire que le cron a raté trois mois d'affilée.

## Résultats

- Purge mensuelle : `DROP` d'une partition, 200 ms au lieu de 3 h de `DELETE`.
- Dump nocturne : 2 h 10 au lieu de 3 h 40.
- Requêtes de la timeline (`WHERE load_id = ? ORDER BY occurred_at`) : inchangées, l'index `idx_load_events_load_id` existe sur chaque partition et le planner fait un `Append` sur les 24 partitions, ce qui coûte 1 ms de plus. Les requêtes du rating ([[carrier-rating-computation]]) ont un filtre sur `occurred_at` et ne touchent que 3 partitions.
- Ce qui s'est dégradé : `SELECT ... WHERE id = ?` sans `occurred_at` scanne les 24 partitions. Il n'y avait qu'un seul endroit qui le faisait, le lien "voir l'événement" dans l'admin, et on lui a ajouté la date.
