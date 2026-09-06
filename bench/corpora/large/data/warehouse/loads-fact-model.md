---
name: loads-fact-model
description: core.loads: one row per load with transition timestamps and 14 derived columns, ReplacingMergeTree, sorted by shipper and posted_at
type: reference
status: active
verified: 2026-06-09
---

`core.loads` est la table centrale de l'entrepôt : une ligne par chargement, état courant et horodatages de chaque transition. Elle est reconstruite toutes les 10 minutes par le modèle `loads` de [[marmot-model-runner]] à partir de `raw.cdc_app_loads`.

## Clé et tri

`ORDER BY (shipper_id, posted_at, load_id)`, partition par mois de `posted_at` (voir [[partitioning-and-ttl]]). Moteur `ReplicatedReplacingMergeTree(version)` avec `version = cdc_ts_ms` : la dernière version CDC gagne, et les lectures font `FINAL` ou passent par la vue `core.loads_current` qui l'ajoute (voir [[dedup-replacing-merge-tree]]).

## Colonnes de transition

Une colonne `Nullable(DateTime64(3, 'UTC'))` par état, remplie avec le premier `ts_ms` où le CDC a vu cet état : `posted_at`, `bidding_closed_at`, `awarded_at`, `picked_up_at`, `delivered_at`, `pod_accepted_at`, `cancelled_at`. Le premier passage compte ; un chargement rouvert garde son `bidding_closed_at` initial et une colonne `reopened_count`.

## Colonnes dérivées

| Colonne | Définition |
|---|---|
| `distance_km` | distance d'itinéraire fournie par l'app à la publication, pas la distance à vol d'oiseau |
| `lane_cluster_id` | cluster pricing du chargement, copié depuis le devis, peut être null pour les chargements d'avant 2025 |
| `origin_nuts3`, `destination_nuts3` | par le dictionnaire `postal_nuts3` ([[geo-dictionaries]]) |
| `bid_count` | nombre d'enchères `open` initiales, calculé depuis `core.bids` |

### Montants et délais

| Colonne | Définition |
|---|---|
| `awarded_amount_cents`, `awarded_currency` | montant de l'enchère acceptée |
| `suggested_amount_cents` | prix suggéré du devis vu par le chargeur |
| `award_ratio` | `awarded_amount_cents / suggested_amount_cents`, null si l'un des deux manque |
| `time_to_first_bid_s` | `min(bids.created_at) - posted_at` |
| `time_to_award_s` | `awarded_at - posted_at` |

### Attributs copiés

| Colonne | Définition |
|---|---|
| `is_cross_border` | pays d'origine différent du pays de destination |
| `adr_class`, `vehicle_type`, `max_price_cents` | copiés de l'app |
| `is_deleted` | CDC `op = 'd'` vu |
| `entity_code` | entité facturante du chargeur |

`award_ratio` est la métrique que pricing suit chaque mois ; sa définition ici est la seule qui fait foi, et le tableau de bord pricing lit `marts.pricing_daily` qui la reprend telle quelle.

## Ce que la table ne contient pas

- Les adresses complètes : `core.loads` a les codes postaux et les NUTS-3, l'adresse est dans `core.load_addresses` avec un accès restreint (données personnelles pour les livraisons chez des artisans).
- Les enchères : dans [[bids-fact-model]].
- Les factures : dans [[invoice-mart]], jointes par `load_id`.

## Pièges

- `delivered_at` est l'heure déclarée par le conducteur dans l'app, pas l'heure du POD. Le POD peut être signé le lendemain. Pour la ponctualité, utiliser `delivered_at` ; pour la facturation, `pod_accepted_at`.
- Les chargements de test (comptes internes, `shipper_id` dans le dictionnaire `internal_accounts`) sont exclus par le modèle. Un compte interne créé sans être ajouté au dictionnaire pollue les chiffres pendant une semaine, c'est arrivé en mars 2026 avec 600 chargements de test de l'équipe dispatch.
