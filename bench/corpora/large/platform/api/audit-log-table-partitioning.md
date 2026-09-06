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

## Requêtes de vérification et ce qu'on a écarté

Les requêtes que l'astreinte et le cron utilisent, dans `docs/sql/partitions.md` :

```sql
-- partitions existantes et bornes
SELECT c.relname, pg_get_expr(c.relpartbound, c.oid) AS bounds,
       pg_size_pretty(pg_total_relation_size(c.oid)) AS size
FROM pg_inherits i JOIN pg_class c ON c.oid = i.inhrelid
WHERE i.inhparent = 'load_events'::regclass
ORDER BY c.relname;

-- la partition par défaut doit être vide
SELECT count(*) FROM load_events_default;

-- lignes par mois, à comparer avec le compte d'avant migration
SELECT date_trunc('month', occurred_at) AS m, count(*)
FROM load_events GROUP BY 1 ORDER BY 1;
```

Chiffres du backfill de janvier 2026, pour référence : 41,2 M de lignes copiées en 6 h 05, 100 000 lignes par tranche, une pause de 200 ms entre deux, 412 tranches. Le compte par mois était identique des deux côtés à la fin, sauf le mois courant qui recevait des écritures par le trigger pendant la copie, et qui a été recompté après le renommage : identique aussi. La copie a fait monter le WAL de 60 Go sur la journée, ce qui a rempli le disque de la réplique à 85 % parce que l'archivage vers le magasin d'objets ne suivait pas au même rythme. La prochaine opération de ce type demandera une vérification de la place libre sur la réplique avant de commencer, et c'est dans la fiche.

Ce qu'on a écarté :

- L'extension de gestion automatique des partitions (`pg_partman`). Elle fait ce que notre commande fait, mais elle demande un worker en arrière-plan et une configuration dans une table de l'extension. Notre commande fait 120 lignes, tourne dans un CronJob qu'on sait surveiller, et n'a pas de dépendance. Si on partitionne une troisième table, on reconsidère.

- Le partitionnement par hash sur `load_id` pour accélérer la timeline d'un chargement. La timeline touche 24 partitions avec un index chacune, 1 ms de plus qu'avant, et un hash aurait perdu la suppression par `DROP` d'un mois, qui était le but.

- Le partitionnement de `loads` elle-même. Elle a 2,2 M de lignes vivantes après archivage, ce n'est pas une table qui a besoin de ça, et le nombre de clés étrangères qui pointent dessus (7) rendrait l'opération pénible pour un gain nul.

Le point qui reste à surveiller : `audit_log` reçoit environ 90 000 lignes par jour et grossit avec le nombre d'utilisateurs, pas avec le nombre de chargements. À 24 mois de rétention, c'est 65 M de lignes en régime établi, contre 28 M aujourd'hui. Les partitions mensuelles feront 2,7 M de lignes chacune, ce que les index tiennent sans problème.
