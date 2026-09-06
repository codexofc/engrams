---
name: search-index-mapping-loads
description: Le mapping loads-v7 : geo_point, keywords d'état et de véhicule, prix en cents et au km, analyseur city, champs stockés non indexés, ordre des déploiements
type: reference
status: active
verified: 2026-07-24
---

# Mapping de l'index `loads-v7`

Le document est construit par `haystack-indexer` depuis la ligne `loads` et quelques jointures ([[search-indexing-pipeline]]). Le mapping est figé en `mappings/loads-v7.json` dans le dépôt de l'indexeur, `dynamic: strict` : un champ inconnu est refusé, pas deviné.

## Champs

### Identité et état

| Champ | Type | Remarque |
|---|---|---|
| `load_id` | keyword | `_id` du document, aussi |
| `version` | long | version externe, `external_gte` |
| `status` | keyword | `OPEN` ou `BIDDING` seulement en `searchable = true` |
| `searchable` | boolean | filtre systématique |
| `visibility` | keyword | `PUBLIC`, `PRIVATE` ; `PARTNER_ONLY` n'est pas indexé |
| `shipper_org_id` | keyword | pour `PRIVATE` (transporteurs favoris) et pour exclure ses propres chargements |
| `source` | keyword | `direct`, `fretzone`, `cargolink` |
| `published_at` | date | fraîcheur pour le classement |
| `bidding_closes_at` | date | filtre « encore ouvert » |
| `indexed_at` | date | affiché au support |
| `terminated_at` | date | pour la purge à 7 jours |

### Lieux et dates

| Champ | Type | Remarque |
|---|---|---|
| `pickup.geo` | geo_point | requête par rayon ([[search-geo-radius-queries]]) |
| `pickup.city` | text, analyseur `city` | plus `pickup.city.raw` keyword |
| `pickup.postcode` | keyword | préfixe possible (`wildcard` évité, on indexe aussi `postcode_2`) |
| `pickup.country` | keyword | facette |
| `pickup.window_start`, `pickup.window_end` | date | UTC |
| `delivery.*` | idem | |
| `distance_km` | integer | calculée à l'indexation depuis les deux points (route estimée par la table de distances du pricing, pas la ligne droite) |

### Marchandise et véhicule

| Champ | Type | Remarque |
|---|---|---|
| `vehicle_types` | keyword | liste, facette |
| `adr_classes` | keyword | liste |
| `weight_kg`, `pallets`, `ldm` | integer / integer / half_float | filtres de gabarit |
| `goods` | text, analyseur `standard` plus synonymes | seul vrai texte libre |
| `temperature_controlled` | boolean | |

### Prix

| Champ | Type | Remarque |
|---|---|---|
| `target_price_cents` | long | dans `target_currency` |
| `target_currency` | keyword | |
| `target_price_eur_cents` | long | converti au taux du jour d'indexation, pour trier et comparer |
| `price_per_km_eur_cents` | integer | calculé : `target_price_eur_cents / distance_km`, null si sur offre |
| `bid_count` | integer | mis à jour à chaque offre (événement `bid.placed`) |
| `lowest_bid_eur_cents` | long | visible du chargeur seulement, indexé pour un futur classement, non renvoyé aux transporteurs |

### Stocké, non indexé

`reference` (celle du chargeur, `index: false`, renvoyée dans les résultats), `shipper_display_name`, `summary` (une ligne pour la carte de résultat). Tout ce qui sert à afficher sans servir à chercher est `index: false` pour garder l'index petit.

## L'analyseur `city`

`standard` puis `lowercase`, `asciifolding` (pour que « Straßburg », « Strasbourg » et « strasburg » se rapprochent) et un filtre de synonymes chargé depuis `synonyms/cities.txt` ([[search-synonyms-city-names]]). Pas de stemming : les noms de ville ne se conjuguent pas et le stemming français cassait « Nantes » en « nant ».

## Ce qui n'est pas dans l'index

L'adresse (rue), les contacts, la valeur de la marchandise, le prix plancher du chargeur, les offres elles-mêmes. Un transporteur ne cherche pas là-dessus et ça ne doit pas fuiter par la recherche. L'API de recherche ne renvoie que les champs listés dans `search-response-fields.yaml`, quel que soit le mapping.

## Changer le mapping

Un champ **ajouté** : mettre à jour le mapping de l'index vivant (`PUT loads-v7/_mapping`), déployer l'indexeur qui le remplit, puis un backfill si les anciens documents doivent l'avoir. Dans cet ordre, sinon l'indexeur envoie un champ que `dynamic: strict` refuse et tout part en rejet ; c'est arrivé une fois, 40 minutes de rejets, le lundi où `price_per_km_eur_cents` a été ajouté avant le mapping.

Un champ **modifié** (type, analyseur) : nouvel index `loads-v<N+1>` avec le nouveau mapping, backfill, bascule des alias ([[search-reindex-runbook]]). Jamais de modification en place.

Un champ **retiré** : il reste dans le mapping jusqu'au prochain nouvel index, l'indexeur cesse de le remplir. Un mapping ne rétrécit pas en place.

## Taille

90 000 documents, 1,2 Go primaire, environ 13 Ko par document dont un tiers pour les champs `text` et leurs positions. Le champ `goods` avec ses positions est le plus gros ; on l'a gardé parce que la recherche de phrase (« palettes europe ») marche mieux avec.

## Historique des versions

- `loads-v1` à `v3` (janvier et février 2026, avant la production) : itérations sur l'analyseur des villes et le format des prix.

- `loads-v4` (mars 2026, mise en production) : le mapping de lancement, sans `price_per_km_eur_cents` ni `bid_count`.

- `loads-v5` (avril 2026) : ajout de `bid_count` et `lowest_bid_eur_cents` pour les mises à jour partielles après l'incident de retard d'indexation.

- `loads-v6` (11 mai 2026) : le routage par pays, retiré trois jours plus tard.

- `loads-v7` (14 mai 2026) : routage par défaut, plus `tail_lift` et `side_loading` ajoutés en juillet par `PUT _mapping` (additif, pas de nouvel index).

Un `loads-v8` est prévu pour le passage des filtres de synonymes en `search_analyzer` sur le champ `goods` (déjà fait pour `city`) et pour retirer trois champs que personne n'interroge (`weight_kg` en `half_float` au lieu de `integer`, deux champs de compatibilité du lancement). Pas de date, ce n'est pas urgent.

## Vérifier un document

Pour voir ce que l'index sait d'un chargement, depuis un port-forward : `GET loads-read/_doc/<load_id>?_source_excludes=lowest_bid_eur_cents`. Le champ exclu est le seul qui ne doit pas apparaître dans un ticket de support, parce qu'il révèle l'offre la plus basse d'un chargement à qui lit le ticket.
