---
name: dq-rules-review-feedback
description: Ce que la revue mensuelle des règles a appris en neuf mois: une règle par hypothèse, l'invariant pas la vraisemblance, sample_keys partout, cinq questions
type: feedback
status: active
verified: 2026-07-07
---

# Revue des règles : ce qu'on a appris

Compte rendu cumulé des revues mensuelles (premier jeudi du mois, une heure, l'équipe données et un représentant par domaine) depuis la mise en place du framework ([[dq-framework-overview]]) en octobre 2025.

## Ce qui tient

- **Une règle par hypothèse, y compris celle qui « ne changera jamais ».** L'hypothèse « tout est en EUR » retirée d'un mart parce que redondante a coûté six semaines de chiffre d'affaires faux ([[missed-2026-04-sek-invoices-summed-as-eur]]). Une hypothèse écrite en règle `accepted_values` coûte une ligne de yaml et échoue avec un message clair le jour où elle devient fausse. Depuis mai, chaque modèle marmot de `mart` liste ses hypothèses dans son en-tête (devise, fuseau, unité, périmètre) et le lint vérifie qu'une règle existe par hypothèse listée.

- **Tester l'invariant, pas la vraisemblance.** Le tableau des comptes de lignes de novembre 2025 ([[missed-2025-11-duplicate-loads-after-replay]]) est au mur. `count(*)` contre hier ne détecte rien d'utile ; `count(*) = count(distinct clé)` détecte le problème en 200 ms.

- **Comparer les lignes entre elles plutôt qu'à l'horloge.** La règle « pas dans le futur » évaluée 15 minutes après n'a rien vu du décalage de juin ; la règle « pas antérieur à la transition précédente » a vu ([[incident-2026-06-timezone-shift-status-history]]). Une règle qui dépend de son heure d'exécution dépend de son ordonnancement, et l'ordonnancement bouge.

- **`sample_keys` sur toute règle de ligne.** La demi-heure gagnée en janvier ([[caught-2026-01-null-carrier-ids]]) se répète à chaque alerte. Une alerte sans clé d'exemple oblige à réécrire la requête de la règle à la main, sous pression.

- **La réconciliation dit que deux systèmes sont d'accord, pas qu'ils ont raison.** La réconciliation Payla ([[reconciliation-payla-settlements]]) était juste pendant l'affaire SEK. Il faut les deux : des réconciliations et des invariants.

- **`page` étroit.** 41 règles `page`, 3 appels en juin, 3 justifiés ([[dq-severity-and-paging]]). Le jour où un appel n'est pas justifié, la règle est rétrogradée à la revue suivante sans discussion.

## Ce qui a changé d'avis

- **Les règles d'anomalie sur les totaux.** Elles ont été écrites sur les totaux (plus simple) et ont manqué février parce que le total noyait le pays. Tout est par segment depuis mars ([[volume-anomaly-seasonal-thresholds]]), et une règle d'anomalie sans segment doit dire pourquoi.

- **`warn` n'est pas « on regardera ».** Le `warn` de février classé « fin de mois » sans requête a laissé 2 140 factures fausses deux semaines de plus. Un `warn` se classe avec une requête qui montre pourquoi c'est normal, collée dans le fil, ou il reste ouvert.

- **Les règles générées ne suffisent pas.** `schema` et `freshness` sur chaque table donnent une impression de couverture. Sur les 62 tables couvertes, 19 n'ont que ces deux règles générées, et aucune d'elles n'a jamais rien attrapé de métier. Le lint `--strict` exige depuis mai une règle `not_null` et une `unique` écrites par le propriétaire pour toute table `tier: 1`.

## Les cinq questions posées à toute nouvelle règle

1. **Quel fait métier devient faux si la règle échoue ?** Si la réponse est « le nombre est bizarre », c'est une anomalie en `warn`, pas un invariant en `page`.

2. **Qui se lève, et que fait-il en premier ?** Le runbook est écrit avant la fusion, avec la première requête.

3. **La règle dépend-elle de son heure d'exécution ?** Si oui, la remplacer par une comparaison entre données (`produced_at`, transition précédente, version).

4. **Que voit l'alerte ?** Des clés d'exemple, ou un lien vers une requête enregistrée pour les agrégats.

5. **Combien coûte-t-elle ?** Une règle de plus de 10 s sur une table de plus de 100 M de lignes passe par la revue de coût ([[dq-checks-runtime-cost]]) avant d'être ajoutée au chemin post-hook.

## Ce qu'on n'a pas encore réglé

- La couverture de `raw` est nulle par choix (ce sont des copies), mais deux des trois incidents manqués auraient été visibles plus tôt avec une règle `unique` sur `raw.cdc_app_loads`. On garde le choix et on note le doute.

- Les règles sur les marts du produit (`mart.product_*`) sont écrites par l'équipe données faute de propriétaire disponible, ce qui contredit le principe. Le produit a promis une personne pour septembre.
