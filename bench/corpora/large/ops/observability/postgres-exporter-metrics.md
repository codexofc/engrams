---
name: postgres-exporter-metrics
description: PostgreSQL metrics come from the CloudNativePG operator's built-in exporter plus custom queries (seq scans per table, bloat estimate, long transactions, replication slots), scraped every 30 s, and PgBouncer from its own exporter
type: reference
status: active
verified: 2026-03-30
---

# Métriques PostgreSQL et PgBouncer

## Source

L'opérateur CloudNativePG expose un exporteur sur chaque instance (port 9187), avec les métriques de base (`cnpg_pg_stat_database_*`, `cnpg_pg_replication_lag`, `cnpg_backends_*`, `cnpg_pg_stat_archiver_*`, `cnpg_backup_*`). Le `PodMonitor` est celui de l'opérateur, intervalle 30 s (15 s doublait la charge des requêtes de catalogue pour rien).

Les métriques custom sont déclarées dans un ConfigMap `postgres-custom-queries` référencé par le `Cluster` (`monitoring.customQueriesConfigMap`), fichier `components/postgres/base/custom-queries.yaml`.

## Requêtes custom (ce qu'elles répondent)

| Métrique | Requête (résumé) | Pourquoi |
|---|---|---|
| `cnpg_table_seq_scan_total{table}` | `pg_stat_user_tables.seq_scan` pour les 20 plus grosses tables | le panneau ajouté après l'incident de recherche de février 2026 : un `Seq Scan` sur `loads` qui apparaît, c'est un plan qui a basculé |
| `cnpg_table_n_dead_tup{table}` et `n_live_tup` | `pg_stat_user_tables` | bloat visible et autovacuum en retard |
| `cnpg_table_last_autoanalyze_age_seconds{table}` | `now() - last_autoanalyze` | l'autre signal du même incident, les stats périmées |
| `cnpg_long_transactions{state}` | `count(*) FROM pg_stat_activity WHERE xact_start < now() - interval '30s'` par état | une transaction longue tient une connexion PgBouncer et bloque les `ALTER` |
| `cnpg_oldest_xact_age_seconds` | `max(now() - xact_start)` | la plus vieille |

| Métrique | Requête (résumé) | Pourquoi |
|---|---|---|
| `cnpg_replication_slot_lag_bytes{slot}` | `pg_replication_slots` | le slot de la plateforme data (décodage logique) qui prend du retard fait grossir le WAL |
| `cnpg_index_size_bytes{index}` | `pg_relation_size` pour les index de plus de 100 Mo | les index GIN et GiST de la recherche, à surveiller après réindexation |
| `cnpg_partitions_count{parent}` | `pg_inherits` | le nombre de partitions de `load_events`, doit avancer d'une par mois |
| `cnpg_partition_default_rows{parent}` | `count(*)` sur la partition par défaut | source de l'alerte `PartitionDefaultNotEmpty` |
| `cnpg_invoice_counter_gap{entity,year}` | `max(number) - count(*)` sur `invoices` finalisées | zéro ou une facture manque, ce qui est un incident comptable, `warn` |

Les requêtes custom tournent avec le rôle `exporter` (lecture seule, `pg_monitor`), et chacune a `target_databases: ["halden"]` et un `cache_seconds` de 30 (60 pour celles sur `pg_stat_user_tables`, qui coûtent). Une requête custom qui prend plus de 500 ms est retirée : l'exporteur ne doit pas être une charge.

## PgBouncer

Exporteur `pgbouncer_exporter` en sidecar des pods `pgbouncer-api` et `pgbouncer-workers`, qui interroge `SHOW POOLS`, `SHOW STATS`, `SHOW LISTS`. Métriques qui comptent :

- `pgbouncer_pools_client_waiting_connections` : source de `PgBouncerClientsWaiting` (`warn` à 5 pendant 2 min).

- `pgbouncer_pools_client_maxwait_seconds` : le plus vieux client en attente.

- `pgbouncer_pools_server_active_connections` / `server_idle` : le taux d'occupation du pool de 60. Au-dessus de 50 actifs pendant 5 minutes, `warn`, c'est le signe qu'on approche du plafond avant que les clients n'attendent.

- `pgbouncer_stats_queries_pooled_total` et `avg_query_time_seconds`.

## Dashboard et alertes

Le dashboard `hf-postgres` (voir [[dashboards-conventions]]) a en rangée "Home" : transactions par seconde, ratio de rollbacks, latence moyenne des requêtes depuis PgBouncer, connexions et lag de réplication. Puis les rangées "Tables" (seq scans, bloat, analyze age), "Verrous et transactions longues", "WAL et archivage", "PgBouncer".

Les alertes `page` sont dans [[alerting-rules-catalogue]] (`PostgresWalArchiveFailing`, `PostgresBackupTooOld`). Les `warn` : `PostgresReplicationLagHigh` (30 s), `PostgresLongTransaction` (5 min), `PostgresSeqScanSpike` (`increase(cnpg_table_seq_scan_total{table="loads"}[10m]) > 50`, ce qui ne devrait jamais arriver sur cette table), `PostgresConnectionsNearMax` (180 sur 200), `PostgresReplicationSlotLagHigh` (2 Go).

## Ce qu'on ne collecte pas

`pg_stat_statements` par requête normalisée : la cardinalité (des milliers de requêtes distinctes) est trop élevée pour Prometheus. On lit `pg_stat_statements` à la main, ou via une tâche horaire qui écrit le top 20 par temps total dans une table `sys_query_stats` que le dashboard interroge via la source de données PostgreSQL de Grafana. C'est ce qui a trouvé la requête en double de `carrier_scores` mentionnée dans [[tracing-otel-collector]].

## Coût de l'exporteur et ce qui a été retiré

Chaque requête custom est chronométrée par l'exporteur lui-même (`cnpg_collector_collection_duration_seconds{collector}`), et le dashboard a un panneau caché "coût de l'exporteur" qui les classe. Valeurs en mai 2026, sur le primaire :

| Collecteur | Durée médiane | Note |
|---|---|---|
| `pg_stat_user_tables` (seq scans, dead tuples, analyze age) | 40 ms | 20 tables, cache 60 s |
| `pg_stat_activity` (transactions longues) | 3 ms | |
| `pg_replication_slots` | 1 ms | |
| `pg_relation_size` des gros index | 12 ms | |
| partitions | 8 ms | `pg_inherits` plus un `count(*)` sur la partition par défaut, qui est vide |
| `invoice_counter_gap` | 25 ms | index sur `(billing_entity, number)`, sinon 400 ms |

Total : environ 90 ms toutes les 30 secondes, soit 0,3 % d'une connexion. Acceptable.

### Ce qui a été retiré

- Une requête de bloat estimé (la fameuse requête à base de `pg_class` et de statistiques par colonne qui traîne dans tous les dépôts d'exemples) : 1,8 s sur notre catalogue, et un chiffre que personne ne savait interpréter. Remplacée par `n_dead_tup / n_live_tup`, qui est grossier mais suffit à voir un autovacuum en retard.

- `pg_stat_statements` par requête normalisée, déjà mentionné, pour la cardinalité.

- Un `count(*)` sur `loads` par statut, 300 ms à cause du parcours d'index sur 2,2 M de lignes : l'application expose déjà `hf_loads_by_status` depuis son propre exporteur, calculé à partir des transitions, sans requête.

La règle qui en découle : une requête custom doit prendre moins de 100 ms sur le primaire de production (mesuré, pas estimé), sinon elle passe sur la réplique (`targetInstance: replica` dans la configuration de l'opérateur, ce qui est possible depuis la version 1.22) ou elle est retirée. Deux requêtes tournent sur la réplique pour cette raison : les tailles d'index et le gap des numéros de facture.

Dernier point, qui a coûté une matinée : l'exporteur se connecte avec `sslmode=require` à l'instance locale via le socket réseau, et après la rotation du certificat de la CA interne en janvier 2026 il a refusé la connexion pendant 20 minutes, jusqu'au redémarrage du pod. Les métriques PostgreSQL étaient absentes, et l'alerte `absent(cnpg_collector_up)` n'existait pas. Elle existe maintenant, en `warn`, `for: 5m`.
