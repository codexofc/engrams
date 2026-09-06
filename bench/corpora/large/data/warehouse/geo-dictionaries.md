---
name: geo-dictionaries
description: Geo and calendar lookups as ClickHouse dictionaries (postal_nuts3, country_calendar, fx_rates), hourly reload, dictGet pitfalls
type: reference
status: active
verified: 2026-01-27
---

Les tables de correspondance géographiques et calendaires sont des dictionnaires ClickHouse dans la base `dict`, chargés depuis des tables `MergeTree` de la même base (`dict.src_*`), elles-mêmes alimentées par des fichiers versionnés dans le dépôt `wh-dictionaries`.

## Dictionnaires

| Dictionnaire | Clé | Attributs | Lignes | Source |
|---|---|---|---|---|
| `dict.postal_nuts3` | `(country, postal_prefix)` | `nuts3`, `lat`, `lon` | 240 000 | table de correspondance de l'office statistique européen, rechargée deux fois par an |
| `dict.nuts3_names` | `nuts3` | `name`, `nuts2`, `nuts1`, `country` | 1 500 | même source |
| `dict.country_calendar` | `(country, date)` | `is_business_day`, `holiday_name` | 40 000 (11 pays × 10 ans) | fichier maintenu à la main, voir plus bas |
| `dict.fx_rates` | `(date, base, quote)` | `rate`, `source` | 30 000 | CDC de `fx_rates` applicative (BCE, NBP, CNB) |
| `dict.internal_accounts` | `account_id` | `kind` | 200 | fichier |
| `dict.lane_clusters` | `lane_cluster_id` | `origin_nuts3`, `destination_nuts3`, `distance_band`, `vehicle_type`, `confidence` | 2 200 | CDC de `lane_clusters` |

### Disposition et rechargement

Disposition `complex_key_hashed` pour les clés composées, `flat` pour `nuts3_names`. `LIFETIME(MIN 3000 MAX 3600)` : rechargement toutes les heures environ, ce qui suffit puisque les sources changent rarement ; `SYSTEM RELOAD DICTIONARY dict.fx_rates` force le rechargement après un backfill de taux.

## Usage

```
SELECT dictGet('dict.postal_nuts3', 'nuts3', (origin_country, substring(origin_postal_code, 1, 5))) AS origin_nuts3
```

Le préfixe postal utilisé varie par pays : 5 caractères en FR, DE, PL (`12-345` réduit à `12345` sans le tiret), 4 en NL, AT, BE, 6 en RO. La fonction `normalizePostal(country, code)` (UDF SQL dans `dict`) fait la normalisation, et [[loads-fact-model]] l'utilise ; ne pas refaire la logique dans une requête.

`dictGetOrDefault` avec une valeur explicite plutôt que `dictGet` seul dans les modèles : un code postal inconnu renvoie `''` sur `nuts3` par défaut, ce qui s'est retrouvé en clé de groupement pendant un mois avant qu'on s'en aperçoive. Le modèle `loads` renvoie désormais `'UNKNOWN'` et un test `accepted_values` compte les `UNKNOWN` (moins de 0,5 % attendu).

## Le calendrier

`country_calendar` est un fichier CSV dans le dépôt, un pays par section, jours fériés nationaux uniquement (pas les régionaux, sauf les Länder allemands pour lesquels on prend l'union, ce qui surestime un peu les jours fériés en Allemagne). Mis à jour chaque octobre pour l'année suivante. Le modèle de prédiction de demande de l'équipe ML lit le même fichier ; c'est la seule source de calendrier autorisée.

## Pièges

- Un dictionnaire se recharge entièrement, pas par différence. `postal_nuts3` à 240 000 lignes prend 4 s, sans incidence.
- Un dictionnaire dont la source est vide au moment du rechargement (table `src_*` en cours de remplacement) reste sur son ancienne version, ce qui est le comportement voulu, mais le journal ne le dit pas fort ; `system.dictionaries.last_exception` est à surveiller.
- Une jointure sur un dictionnaire dans une vue matérialisée voit l'état du dictionnaire à l'insertion ([[materialized-views-pitfalls]]).
