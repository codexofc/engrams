---
name: toll-cost-tables
description: Toll estimates from toll_segments per country, vehicle and Euro class, quarterly refresh, German CO2 component as a separate column
type: project
status: active
verified: 2026-01-15
---

La majoration `toll` (règle 7 du [[surcharge-rules-catalog]]) est un montant fixe estimé à partir de l'itinéraire calculé par le service de routage et de la table `toll_segments`.

## Table

`toll_segments` : `country`, `segment_id`, `length_km`, `vehicle_class` (`n2`, `n3_2axles`, `n3_3axles`, `n3_4plus`), `euro_class` (`euro5`, `euro6`), `rate_eur_per_km`, `co2_component_eur_per_km`, `valid_from`, `valid_to`. Environ 38 000 segments pour FR, DE, PL, NL, BE, AT, CZ, ES, IT.

Le routage renvoie la liste des segments traversés ; `pricing-svc` fait la somme de `length_km × (rate + co2_component)` pour la classe de véhicule du chargement (déduite du type de véhicule et du poids déclaré, par défaut `n3_4plus` et `euro6`).

## Sources et rafraîchissement

- Allemagne : grille officielle de la Maut, avec la composante CO2 introduite le 1er décembre 2023, qui a presque doublé le péage des Euro 6 (de 0,19 à 0,348 EUR/km pour un 5 essieux). On la garde en colonne séparée parce que les chargeurs allemands demandent la décomposition sur leurs propres factures.
- France : grilles des concessionnaires, classe 4 pour tous nos véhicules. Hausse annuelle au 1er février, saisie dans la semaine qui suit.
- Pologne : système national, tarif par classe d'émission.
- Autriche : le plus cher au km (0,45 EUR/km en moyenne pour un 4 essieux Euro 6), et le Brenner est un cas à part avec un tarif de nuit.

Rafraîchissement trimestriel par pricing, avec `pricing-svc toll import <country> <file.csv>`, qui crée une nouvelle plage `valid_from` sans toucher aux anciennes lignes. Un devis conserve la version de table utilisée (`config_version`).

## Précision

Comparé aux relevés de péage réels remontés par 60 transporteurs volontaires en 2025 (2 100 trajets) : erreur médiane de 4 %, p90 de 14 %. Les écarts viennent surtout des itinéraires réels différents de l'itinéraire calculé (le conducteur évite un tronçon, prend une nationale). On n'essaie pas de faire mieux : le péage est une estimation dans le devis, le transporteur reste libre de son enchère.

## Ferries et tunnels

Les traversées (Calais-Douvres, Rostock-Trelleborg, etc.) sont dans `ferry_rates` et relèvent de la règle 8. Les tunnels à péage (Mont-Blanc, Fréjus) sont des segments de `toll_segments` avec un `length_km` fictif pour que la formule donne le tarif forfaitaire ; ce n'est pas élégant mais ça évite un cas particulier dans le moteur. Les interdictions de tunnel pour l'ADR sont gérées par le routage, voir [[adr-surcharge-hazmat]].
