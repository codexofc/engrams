---
name: surcharge-rules-catalog
description: The nine surcharge rules of pricing-svc, their order, which stack and which do not, and where each rate is configured
type: reference
status: active
verified: 2026-06-15
---

Les majorations sont calculées par `pricing-svc` dans `surcharges/engine.py`, appliquées sur le prix de base de la ligne (voir [[pricing-lane-clusters]]) et renvoyées détaillées dans le devis pour que le transporteur et le chargeur voient la décomposition. Contexte général : [[bid-engine-architecture]].

## Ordre d'évaluation

Les règles sont évaluées dans l'ordre ci-dessous. Les majorations en pourcentage s'appliquent toutes sur le prix de base, pas en cascade ; les montants fixes s'ajoutent à la fin. Ce choix vient de l'expérience [[weekend-surcharge-backfire]].

### Table des règles

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

## Détail des règles qui posent question

**`night`.** Le créneau de chargement est celui déclaré par le chargeur, en heure locale du lieu de chargement (le fuseau vient des coordonnées, pas du pays du chargeur ; un chargeur français avec un entrepôt à Lisbonne a un créneau en heure de Lisbonne). Un créneau qui chevauche 22:00 (par exemple 20:00 à 23:00) compte comme nuit si plus de la moitié du créneau est après 22:00. Cette règle de moitié est dans `night.py` avec un test sur 21:00 à 23:00 (nuit) et 20:00 à 23:00 (jour).

**`urgent`.** Calculé à l'émission du devis, pas à la publication du chargement : un chargement publié 6 h avant le chargement mais consulté par un transporteur 1 h 30 avant produit un devis `urgent` pour ce transporteur. Deux transporteurs voient donc des devis différents sur le même chargement, ce qui a surpris le support en 2025 et est documenté dans la macro `pricing-urgent`.

**`temperature`.** Le +10 % isotherme sans groupe a été ajouté en février 2026 (HF-2445) à la demande des transporteurs de produits secs sensibles (chocolat, pharmacie) qui refusaient les chargements « frigo » à +18 % que les chargeurs sur-déclaraient. Le chargeur choisit maintenant entre `refrigerated`, `insulated` et `ambient`, et 14 % des anciens `refrigerated` sont passés en `insulated`.

**`multi_stop`.** 45 EUR par arrêt au-delà du deuxième (le chargement et la livraison sont les deux premiers). Un chargement avec 3 livraisons a donc 2 arrêts facturés, 90 EUR. Le montant est le même dans tous les pays, en EUR converti au taux du jour pour les devis en PLN ou CZK.

## Versionnage des tables de configuration

Chaque table de majoration (`adr_classes`, `toll_segments`, `ferry_rates`) et chaque constante (`SURCHARGE_*`) est portée par `pricing_config_versions` : une ligne par changement avec `version`, `changed_at`, `changed_by`, `diff jsonb`. Un devis stocke `config_version` ; `pricing-svc quote --replay <quote_id>` recharge la version et recalcule. La CI rejoue 200 devis d'août 2025 à chaque merge request et vérifie l'égalité au cent ([[weekend-surcharge-backfire]]), ce qui prend 4 secondes.

Un changement de constante passe par le back-office pricing, jamais par une variable d'environnement modifiée à chaud : les `SURCHARGE_*` du tableau sont les noms dans la table de configuration, pas des variables d'environnement, malgré leur allure. Ce point est répété parce que deux personnes ont cherché où était définie la variable dans le déploiement.

## Chiffres de juin 2026

Part des devis avec chaque majoration : `fuel` 100 %, `toll` 91 %, `weekend` 9,4 %, `night` 6,1 %, `temperature` 12 %, `adr` 3,1 %, `urgent` 4,8 %, `ferry` 1,2 %, `multi_stop` 7,3 %. Majoration totale médiane en pourcentage : 4,3 % (le carburant seul) ; p90 : 22 %. Le plafond de +60 % a été atteint 31 fois dans le mois.
