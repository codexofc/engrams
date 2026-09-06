---
name: marmot-model-runner
description: marmot, the in-house SQL model runner: YAML headers, a DAG, incremental or full materialisation, tests, backfill by partition
type: reference
status: active
verified: 2026-04-16
---

`marmot` est l'outil interne qui joue les modèles SQL de l'entrepôt. Il ressemble aux outils de transformation du marché mais tient en 4 000 lignes de Python et fait exactement ce qu'on utilise : un DAG de fichiers SQL, deux matérialisations, des tests, un backfill par partition.

## Un modèle

Un fichier `models/<db>/<name>.sql` avec un en-tête YAML :

```
-- name: core.bids
-- materialization: incremental
-- partition_by: toYYYYMM(created_at)
-- unique_key: bid_id
-- depends_on: raw.cdc_app_bids, raw.cdc_app_quotes, dict.fx_rates
-- schedule: every_10_min
-- tests: not_null(bid_id), unique(bid_id), accepted_values(state, [...])
SELECT ...
```

Le SQL référence ses dépendances par leur nom complet ; marmot construit le DAG en lisant les en-têtes, pas le SQL. Une dépendance non déclarée mais utilisée fait échouer `marmot lint` (il compare les tables citées dans le SQL et l'en-tête).

## Matérialisations

- `full` : `CREATE TABLE ... AS SELECT` dans une table temporaire puis `EXCHANGE TABLES`. Pour les dimensions et les petits marts.
- `incremental` : sélectionne les partitions touchées depuis le dernier run (par `max(cdc_ts_ms)` mémorisé dans `marmot._runs`), recalcule ces partitions entières et les remplace par `REPLACE PARTITION`. Pas de `INSERT` incrémental ligne à ligne : recalculer une partition mensuelle de `core.bids` prend 40 s et évite toute logique de fusion. Les partitions anciennes touchées par un événement en retard sont recalculées aussi, voir [[late-arriving-events]].

## Ordonnancement

Le scheduler lance `marmot run --schedule every_10_min` toutes les 10 minutes et `--schedule hourly` à H+05. Un run traite les modèles dans l'ordre du DAG, en parallèle jusqu'à 4 sur des branches indépendantes. Un modèle en échec bloque ses descendants et pas le reste ; l'alerte part au deuxième échec consécutif. La durée d'un run `every_10_min` est de 3 à 6 minutes ; au-delà de 9 minutes, le run suivant est sauté et l'alerte `marmot.run_overlap` se déclenche.

## Tests

`marmot test` après chaque run, sur les partitions recalculées seulement. Tests disponibles : `not_null`, `unique`, `accepted_values`, `relationship` (clé étrangère vers un autre modèle), `row_count_delta` (variation du nombre de lignes par rapport à la veille dans une fourchette). Un test en échec n'annule pas la partition écrite, il alerte ; on a choisi de préférer une donnée fraîche et signalée à une donnée absente, sauf pour `marts.invoice_mart` qui a un test bloquant parce que la finance le lit (voir [[invoice-mart]]).

## Commandes

- `marmot run [--schedule X | --model core.bids] [--full-refresh]`
- `marmot test [--model ...]`
- `marmot backfill --model core.bids --from 2025-11 --to 2026-01`, voir [[backfill-runbook]]
- `marmot lint`, en CI
- `marmot dag --model marts.pricing_daily` affiche les ancêtres et descendants

## Ce qu'on n'a pas fait

Pas de macros ni de templating au-delà de `{{ partition }}` et `{{ last_run }}`. La première version avait un moteur de templates complet et les modèles étaient devenus illisibles ; on l'a retiré en janvier 2026 (HF-2420) et réécrit les 12 modèles qui s'en servaient en SQL plat. Les conventions d'écriture des modèles sont dans [[warehouse-team-preferences]].

## Tables de suivi

- `marmot._runs` : une ligne par exécution de modèle avec `model`, `kind` (`scheduled`, `backfill`, `manual`), `partitions` (liste), `started_at`, `finished_at`, `rows_written`, `status`, `error`. C'est ce que le tableau de bord data lit pour la fraîcheur des modèles et ce que `marmot backfill` consulte pour éviter deux backfills concurrents.
- `marmot._deferred` : partitions à recalculer au passage hebdomadaire, avec la raison (`late_event`, `test_failed`, `manual`).
- `marmot._tests` : un résultat par test et par partition, conservé 90 jours.

Les trois tables sont dans la base `marmot`, `MergeTree`, avec un TTL d'un an sur `_runs`.

## Tests, détails

- `not_null(col)` et `unique(col)` sont évalués sur la partition recalculée seulement ; `unique` sur une table `ReplacingMergeTree` lit avec `FINAL`, sinon le test échoue sur les doublons attendus avant fusion ([[dedup-replacing-merge-tree]]).
- `relationship(col, other_model.col)` vérifie que chaque valeur existe dans l'autre modèle, avec une tolérance de 0,1 % pour les faits dont la dimension est en retard (un chargement d'un transporteur créé il y a 5 minutes et pas encore dans `core.carriers` reconstruit à l'heure).
- `row_count_delta(min, max)` compare le nombre de lignes de la partition à la même partition la veille ; les bornes par défaut sont 0,7 et 1,5, et les modèles de faits les resserrent à 0,9 et 1,2. Ce test est celui qui a signalé le jour de doublons de janvier avant que quelqu'un lise le message de réconciliation ([[duplicate-bids-incident-2026-01]]), mais son alerte n'était pas non plus un appel.
- Un test personnalisé est un fichier SQL dans `tests/<model>/<name>.sql` qui doit renvoyer zéro ligne ; `no_pii_columns` et `totals_match_billing` sont écrits comme ça.

## Ce que marmot ne sait pas faire, volontairement

- Pas de `snapshot` à la manière des outils du marché : la dimension à versions ([[carrier-dimension-scd]]) est un modèle `full` ordinaire qui lit l'historique CDC. Un mécanisme de snapshot recréerait un historique à partir de l'état courant, ce qui est moins bon que l'historique qu'on a déjà.
- Pas de « seeds » (CSV chargés par l'outil) : les fichiers de référence passent par les dictionnaires ([[geo-dictionaries]]) et un dépôt à part.
- Pas d'exécution d'un modèle sur un autre entrepôt qu'un ClickHouse ; le SQL est du ClickHouse sans couche de traduction, et c'est ce qui rend les modèles lisibles.

## Performance d'un run

Un run `every_10_min` de juin 2026 : 14 modèles, 3 min 50 s en médiane, dont `core.bids` 40 s, `core.loads` 25 s, `core.invoices` 20 s, `core.payments` 15 s, `core.load_events` 55 s (le plus gros, 300 M lignes sur la fenêtre), le reste sous 10 s chacun. Le parallélisme à 4 branches économise environ 1 min 30 s par rapport au séquentiel. Le run `hourly` : 9 modèles, 6 min, dont `marts.invoice_mart` 3 min et `marts.bids_enriched` 1 min 40 s.
