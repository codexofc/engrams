---
name: freshness-slo
description: Freshness targets per layer (raw 2 min, core 15, marts 75, invoice_mart 90), measured by the probe, 99.5 % attainment, April 2026
type: project
status: active
verified: 2026-05-06
---

## Objectifs

| Couche | Fraîcheur cible | Mesure | Attente mensuelle |
|---|---|---|---|
| `raw.*` | 2 minutes | `now() - max(received_at)` par table, sondé toutes les 60 s | 99,5 % des sondes |
| `core.*` faits | 15 minutes | `now() - max(cdc_ts_ms)` des lignes présentes, comparé à `raw` | 99,5 % |
| `marts.*_daily`, `bids_enriched` | 75 minutes | `now() - computed_at` | 99 % |
| `marts.invoice_mart` | 90 minutes | idem | 99 %, mais un run bloqué par le test de totaux ([[invoice-mart]]) compte comme respecté si l'alerte est partie |
| `core.carriers` | 90 minutes | `now() - built_at` | 99 % |

La sonde `wh-freshness` écrit dans `admin.freshness_probes` et le tableau de bord data affiche chaque table avec sa fraîcheur courante. Le bandeau « données partielles » du tableau de bord dispatch se déclenche sur la même mesure pour `raw.driver_positions` au-delà de 300 s.

## Pourquoi ces valeurs

- `raw` à 2 minutes : le lot d'ingestion est de 5 s ([[ingestion-kafka-to-clickhouse]]), 2 minutes absorbent un rééquilibrage Kafka ou un redémarrage de `ingest-svc`.
- `core` à 15 minutes : un run marmot toutes les 10 minutes, qui dure 3 à 6 minutes ([[marmot-model-runner]]). Un run sauté pour chevauchement fait dépasser 15 minutes ; c'est le cas qu'on veut voir.
- `marts` à 75 minutes : horaire à H+05, plus la durée du run. Passer les marts à 10 minutes coûterait 4 fois le calcul pour des tableaux de bord que personne ne regarde plus d'une fois par heure ; on a demandé.

## Avril 2026

- `raw` : 99,91 %. Les manquements : le rééquilibrage du 2026-04-09 (7 minutes) et une reprise d'`ingest-svc` le 2026-04-22.
- `core` : 99,63 %. 11 runs sautés dans le mois, tous entre 08:00 et 09:00, quand la partition du mois courant de `core.bids` est la plus grosse et que le pic d'ingestion tombe en même temps. Piste : découper le run de 10 minutes en deux ordonnancements, `core.loads` et `core.bids` d'un côté, le reste de l'autre, ticket HF-2588.
- `marts` : 99,4 %, objectif de 99 % tenu.
- `invoice_mart` : 100 %, aucun blocage ce mois-ci.

## Ce que la fraîcheur ne dit pas

Une table fraîche peut être fausse (voir [[duplicate-bids-incident-2026-01]], où tout était frais et à 31 % de trop). La fraîcheur est une des trois mesures de qualité affichées ; les deux autres sont l'écart de réconciliation Kafka contre `raw` (doit être 0) et le résultat des tests marmot du dernier run.

## Quand l'objectif est manqué

L'astreinte data est paginée seulement sur `raw` au-delà de 10 minutes (un problème d'ingestion se dégrade vite). Les autres couches font une alerte de canal, traitée le matin. La revue mensuelle de la fraîcheur est à la réunion d'équipe du premier mardi, avec cette note mise à jour.
