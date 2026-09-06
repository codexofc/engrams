---
name: disk-full-incident-2025-10
description: Incident of 2025-10-21: ch-w-1a disk full from part explosion and Keeper on data nodes, ingestion stopped 3 h 40, four changes
type: project
status: active
verified: 2025-11-14
---

## Chronologie

- 2025-10-21 06:50 CET : `ch-w-1a` atteint 97 % d'occupation disque. L'alerte à 90 % avait été acquittée la veille par quelqu'un qui pensait à un pic de fusion temporaire.

- 07:15 : les insertions sur le shard 1 échouent avec `Not enough space`. `ingest-svc` retente indéfiniment, le lag Kafka monte. Le shard 2 continue, donc la moitié des données arrive et les tableaux de bord affichent des chiffres à moitié faux, ce qui est pire que rien.

- 07:30 : Keeper, qui tournait sur les nœuds de données à l'époque, n'arrive plus à écrire son journal sur `ch-w-1a`. Le quorum tient avec les deux autres, mais la réplication de `ch-w-1b` vers `ch-w-1a` est bloquée, et `ch-w-1b` commence à accumuler des parts non répliquées.

- 08:00 : diagnostic. `system.parts` sur `raw.product_events` montre 41 000 parts actives pour 400 partitions quotidiennes. Trois vues matérialisées sur cette table écrivent chacune dans leur cible à chaque insertion, avec des blocs plus petits, donc encore plus de parts. Le planificateur de fusions est en retard de plusieurs heures ; les parts non fusionnées occupent 2,5 fois la taille finale.

- 08:20 : suppression des trois vues matérialisées (voir [[materialized-views-pitfalls]]), `DROP PARTITION` des 30 partitions les plus anciennes de `raw.product_events` (au-delà de la rétention de l'époque, elles auraient dû être parties, mais le TTL ne s'applique qu'à la fusion et les fusions étaient en retard).

- 09:10 : 71 % d'occupation, insertions reprises. Rattrapage du lag Kafka jusqu'à 10:30. Durée de l'arrêt d'ingestion sur le shard 1 : 3 h 40.

- Les jours suivants : `OPTIMIZE ... FINAL` partition par partition la nuit, 41 000 parts ramenées à 2 100.

## Causes

1. Partitions quotidiennes sur `raw.product_events` : 400 partitions au lieu de 14, chacune avec ses parts. Passage au mois dans [[partitioning-and-ttl]].
2. Trois vues matérialisées multipliant les insertions.
3. Keeper sur les nœuds de données : un disque plein sur un nœud de données a dégradé la coordination de tout le cluster. Keeper est depuis sur trois nœuds dédiés ([[clickhouse-cluster-layout]]).
4. L'alerte à 90 % acquittable sans justification.

## Changements

- Partitionnement mensuel de toutes les tables `raw` (migration 0142, avec recréation des tables et rechargement depuis Kafka sur 30 jours, le reste accepté comme perte de l'historique brut au-delà, `core` étant intact).
- Suppression des vues matérialisées non justifiées, 14 sur 16.
- Keeper dédié, livré le 2025-11-05.
- Alerte disque à 80 % non acquittable sans un ticket lié, et à 90 % page l'astreinte data. Métrique `wh.parts_per_partition_max` avec alerte à 250.
- Le tableau de bord dispatch affiche désormais un bandeau « données partielles » quand `ingest.lag_seconds` dépasse 300 s sur un shard, pour qu'un chiffre à moitié faux soit visible comme tel.

## Ce qu'on n'a pas fait

Ajouter du disque. On est passé de 4 TB à 4 TB : le problème n'était pas le volume utile (2,1 TB de données réelles) mais les parts non fusionnées. L'occupation est retombée à 58 % après les fusions et se tient entre 60 et 72 % depuis.
