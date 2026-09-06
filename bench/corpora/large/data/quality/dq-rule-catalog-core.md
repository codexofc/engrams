---
name: dq-rule-catalog-core
description: Les règles de qualité sur core.loads, bids, invoices, carriers et load_status_history en juin 2026, avec sévérité et origine; les 41 règles page
type: reference
status: active
verified: 2026-06-26
---

# Catalogue des règles sur les tables `core`

Extrait de `dq/rules/` ([[dq-framework-overview]]) pour les cinq tables qui portent le métier. Les autres tables `core` ont surtout des règles `schema` et `freshness` générées. Sévérités : `page` appelle le propriétaire, `warn` écrit dans son canal, `info` n'apparaît que sur le tableau de bord ([[dq-severity-and-paging]]).

## `core.loads`

Propriétaire `dispatch`, 3,2 M de lignes, une par chargement, dernière version.

### Règles

| Règle | Type | Sévérité | Origine |
|---|---|---|---|
| `loads_pk_unique` | `unique (load_id)` | page | [[missed-2025-11-duplicate-loads-after-replay]] |
| `loads_assigned_have_carrier` | `not_null carrier_id` si statut assigné ou après | page | [[caught-2026-01-null-carrier-ids]] |
| `loads_status_accepted` | `accepted_values status` dans 9 valeurs | page | |
| `loads_pickup_before_delivery` | requête : `pickup_window_start < delivery_window_end` | warn | 2025-12, 40 chargements de test |
| `loads_distance_range` | requête : `distance_km BETWEEN 1 AND 4500` | warn | reprise de la borne physique du feature store ML |
| `loads_country_in_dim` | `accepted_values pickup_country, delivery_country` dans `dim.countries` | warn | |
| `loads_fresh_15m` | `freshness updated_at` 15 min | page | générée |
| `loads_schema` | `schema` | warn | générée |
| `loads_count_vs_cdc` | `reconciliation` : lignes de `core.loads` = clés distinctes de `raw.cdc_app_loads` | page | 2025-11 |

## `core.bids`

Propriétaire `pricing`, 41 M de lignes, une par version d'enchère.

### Règles

| Règle | Type | Sévérité | Origine |
|---|---|---|---|
| `bids_pk_unique` | `unique (bid_id, version)` | page | incident doublons d'enchères de janvier 2026 (côté entrepôt) |
| `bids_amount_positive` | requête : `amount_cents > 0` | page | message empoisonné de février 2026 sur le bus |
| `bids_amount_ceiling` | requête : `amount_cents <= 50 000 000` | warn | même origine |
| `bids_currency_in_dim` | `accepted_values currency` | page | [[missed-2026-04-sek-invoices-summed-as-eur]] |
| `bids_load_exists` | requête : `load_id` présent dans `core.loads` | warn | |
| `bids_withdrawn_have_reason` | `not_null withdraw_reason` si `status = 'WITHDRAWN'` | info | |
| `bids_fresh_15m` | `freshness` | page | générée |
| `bids_daily_volume` | `anomaly` : enchères par jour et par pays d'enlèvement | warn | [[volume-anomaly-seasonal-thresholds]] |

## `core.invoices`

Propriétaire `billing`, 2,1 M de lignes.

### Règles

| Règle | Type | Sévérité | Origine |
|---|---|---|---|
| `invoices_pk_unique` | `unique (invoice_id)` | page | |
| `invoices_currency_not_null` | `not_null currency` | page | avril 2026 |
| `invoices_amount_eur_consistent` | requête : `amount_eur_cents = amount_cents × taux du jour ± 1` | page | [[missed-2026-04-sek-invoices-summed-as-eur]] |
| `invoices_status_transitions` | requête : pas de `paid` sans `issued` antérieur dans l'historique | warn | |
| `invoices_due_after_issued` | requête : `due_date >= issued_at::date` | warn | |
| `invoices_overdue_daily_by_country` | `anomaly` par pays | warn | rejeu des factures de mars 2026 (côté bus) |
| `invoices_vs_payla_settlements` | `reconciliation` quotidienne | page | [[reconciliation-payla-settlements]] |
| `invoices_fresh_30m` | `freshness` | warn | générée, 30 min parce que la facturation est en lots |

## `core.carriers`

Propriétaire `product`, 71 k lignes, dimension à historique.

### Règles

| Règle | Type | Sévérité | Origine |
|---|---|---|---|
| `carriers_one_current_row` | requête : exactement une ligne `is_current` par `carrier_id` | page | 2025-10, dimension à double version courante |
| `carriers_kyc_status_accepted` | `accepted_values kyc_status` dans 5 valeurs Verifid | warn | |
| `carriers_country_in_dim` | `accepted_values` | warn | |
| `carriers_no_pii_columns` | `schema` : liste explicite, échec si une colonne inattendue apparaît | page | filtre de colonnes CDC sur `users`, étendu ici |
| `carriers_fresh_1h` | `freshness` | warn | générée |

## `core.load_status_history`

Propriétaire `dispatch`, 210 M de lignes, une par transition.

### Règles

| Règle | Type | Sévérité | Origine |
|---|---|---|---|
| `lsh_pk_unique` | `unique (load_id, status, changed_at)` | page | |
| `lsh_changed_at_not_future` | requête : `changed_at <= now() + 5 min` | page | [[incident-2026-06-timezone-shift-status-history]] |
| `lsh_changed_at_monotonic` | requête : pas de transition antérieure à la précédente de plus de 1 h pour un même chargement | warn | même origine, la règle qui a attrapé |
| `lsh_status_accepted` | `accepted_values` | page | |
| `lsh_vs_domain_events` | `reconciliation` horaire : transitions dans `core` = messages `domain.load.status-changed` | warn | 2025-11 |
| `lsh_fresh_15m` | `freshness` | page | générée |

## Ce que le catalogue dit de lui-même

Sur les 41 règles `page` de tout le catalogue, 24 sont sur ces cinq tables, et 14 des 24 ont un ticket d'origine : elles existent parce que quelque chose a manqué ou a failli manquer. Les règles sans origine sont celles qu'on aurait écrites de toute façon (clé primaire, valeurs acceptées, fraîcheur). La proportion est stable depuis l'automne 2025 et c'est ce que la revue mensuelle ([[dq-rules-review-feedback]]) regarde en premier : une règle qui n'a jamais rien attrapé et n'a pas d'origine est candidate à `info` ou à la suppression ([[dq-checks-runtime-cost]]).
