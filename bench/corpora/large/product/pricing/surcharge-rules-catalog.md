---
name: surcharge-rules-catalog
description: The nine surcharge rules applied by pricing-svc, their order of evaluation, which ones stack and which do not, and where each rate is configured
type: reference
status: active
verified: 2026-06-15
---

Les majorations sont calculées par `pricing-svc` dans `surcharges/engine.py`, appliquées sur le prix de base de la ligne (voir [[pricing-lane-clusters]]) et renvoyées détaillées dans le devis pour que le transporteur et le chargeur voient la décomposition. Contexte général : [[bid-engine-architecture]].

## Ordre d'évaluation

Les règles sont évaluées dans l'ordre ci-dessous. Les majorations en pourcentage s'appliquent toutes sur le prix de base, pas en cascade ; les montants fixes s'ajoutent à la fin. Ce choix vient de l'expérience [[weekend-surcharge-backfire]].

| # | Code | Type | Valeur | Configuration |
|---|---|---|---|---|
| 1 | `fuel` | % | indice mensuel, voir [[fuel-surcharge-index]] | `fuel_index` |
| 2 | `weekend` | % | +12 % si chargement ou livraison samedi ou dimanche | `SURCHARGE_WEEKEND_PCT` |
| 3 | `night` | % | +8 % si créneau de chargement entre 22:00 et 06:00 heure locale | `SURCHARGE_NIGHT_PCT` |
| 4 | `adr` | % | +15 à +35 % selon classe, voir [[adr-surcharge-hazmat]] | table `adr_classes` |
| 5 | `temperature` | % | +18 % frigo, +10 % isotherme sans groupe | `SURCHARGE_TEMP_*` |
| 6 | `urgent` | % | +20 % si `bidding_closes_at` à moins de 2 h du chargement | `SURCHARGE_URGENT_PCT` |
| 7 | `toll` | fixe | péages estimés sur l'itinéraire, voir [[toll-cost-tables]] | table `toll_segments` |
| 8 | `ferry` | fixe | tarif de la traversée si l'itinéraire en contient une | table `ferry_rates` |
| 9 | `multi_stop` | fixe | 45 EUR par arrêt au-delà du deuxième | `SURCHARGE_STOP_EUR` |

## Cumul

- `weekend` et `night` ne se cumulent pas : on prend le maximum des deux. Un chargement le samedi à 23:00 prend +12 %, pas +20 %.
- `urgent` se cumule avec tout, c'est la seule majoration qui dépend du comportement du chargeur et non du transport.
- `adr` et `temperature` se cumulent (un frigo ADR existe, classe 3 en citerne réfrigérée par exemple).
- Plafond global des majorations en pourcentage : +60 % du prix de base. Atteint sur 0,3 % des devis, presque tous ADR classe 1 le week-end.

## Ce que le transporteur peut modifier

Le devis est une suggestion. Le transporteur enchérit le montant qu'il veut, mais le formulaire pré-remplit la décomposition, et une enchère qui supprime `toll` ou `ferry` déclenche un avertissement (« Votre offre ne couvre pas les péages estimés de 84 EUR »). Depuis mars 2026 l'avertissement a réduit de 40 % les litiges post-livraison sur les péages.

## Historisation

Chaque devis est écrit dans `quotes` avec la liste des majorations appliquées, leur valeur et la version des tables (`config_version`). Les tables de configuration sont versionnées : une modification crée une version, et un devis peut toujours être recalculé à l'identique avec `pricing-svc quote --replay <quote_id>`.
