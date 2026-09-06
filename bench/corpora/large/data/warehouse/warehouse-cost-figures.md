---
name: warehouse-cost-figures
description: Warehouse cost in Q1 2026: 14 200 EUR per month by item, 2.3 EUR per million rows, and the two measures that cut 18 % from Q4
type: project
status: active
verified: 2026-04-21
---

## Coût mensuel, T1 2026 (moyenne des trois mois)

| Poste | EUR/mois | Note |
|---|---|---|
| calcul (4 nœuds données + 3 Keeper + 2 ingest) | 9 100 | réservation annuelle depuis janvier, sinon 12 400 |
| stockage chaud NVMe | 2 300 | 4 × 4 TB |
| stockage froid objet | 1 400 | 31 TB, dont 9 TB de `raw` de plus de 90 jours |
| part Kafka (topics consommés par l'entrepôt) | 900 | refacturation interne de la plateforme |
| sauvegardes (35 jours) | 500 | snapshots incrémentaux vers le stockage objet |
| **total** | **14 200** | |

Rapporté au volume : 6,1 milliards de lignes ingérées par trimestre, soit 2,3 EUR par million de lignes. Rapporté à l'usage : 6 100 requêtes analystes par jour, 0,08 EUR par requête, tout compris.

## Ce qui a baissé depuis le T4 2025 (17 300 EUR)

1. **Réservation annuelle** des nœuds données (−3 300). Décidée après [[disk-full-incident-2025-10]], quand il est devenu clair que la topologie ne changerait pas dans l'année.
2. **Rétention `raw` de 540 à 400 jours** (−600 sur le froid). 540 jours ne servait à personne : le plus vieux backfill depuis `raw` remonte à 11 mois. Voir [[retention-rules]].
3. Les 14 vues matérialisées supprimées ont réduit l'écriture disque de 40 %, sans effet direct sur la facture mais sur la marge de capacité (60 à 72 % d'occupation au lieu de 85 à 97 %).

Le poste qui monte : le stockage froid, +80 GB par semaine. À ce rythme, 45 TB fin 2026, soit +600 EUR par mois. Acceptable ; la prochaine mesure si besoin est la compression `ZSTD(3)` au lieu de `LZ4` sur les colonnes `payload` de `raw` (test sur une partition : −38 % de taille, +15 % de temps de lecture sur le froid, ce qui est indolore puisque le froid est lent de toute façon).

## Ce qu'on ne fera pas

- Réduire à une réplique par shard pour économiser 4 500 EUR : une panne de nœud arrêterait la moitié de l'entrepôt, et l'incident d'octobre a montré ce que ça coûte en confiance.
- Un troisième shard : la charge de calcul est à 35 % en moyenne et 70 % au pic du matin. Un shard de plus se justifiera quand le pic dépassera 85 % de façon répétée, pas avant.

## Refacturation

Le coût est réparti entre les équipes qui lisent l'entrepôt au prorata des requêtes (`system.query_log`, par rôle) : produit 48 %, finance 21 %, ML 19 %, dispatch 12 %. La répartition sert au budget annuel, personne n'est facturé réellement.
