---
name: legacy-cost-spreadsheet
description: Jusqu'en octobre 2025 les coûts vivaient dans une feuille de calcul trimestrielle sans amortissement ni répartition par service; remplacée par costs.yaml
type: reference
status: archived
superseded_by: [[infra-cost-overview-2026]]
verified: 2025-10-15
---

# La feuille de calcul des coûts (archivée)

Ce qui servait de suivi des coûts avant octobre 2025, pour comprendre pourquoi les chiffres d'avant cette date ne se comparent pas directement à ceux de [[infra-cost-overview-2026]].

## Ce que c'était

Un classeur partagé, un onglet par trimestre, une ligne par facture, tenu par la responsable plateforme quand elle y pensait, c'est-à-dire avant chaque revue budgétaire. Les colonnes : fournisseur, montant, commentaire. Pas de quantité, pas de lien vers l'usage, pas de service.

## Ce qui manquait

- **L'amortissement du matériel n'y était pas.** Les serveurs étaient des dépenses d'investissement traitées par la finance dans un autre fichier ; la feuille ne montrait que ce qui arrivait par facture mensuelle. Le coût de l'infrastructure « à la revue » était donc de 55 000 EUR par mois, alors que le coût complet était de 100 000. Toute comparaison avec un devis cloud, qui inclut le matériel, était faussée en faveur du cloud de 45 %, et il a fallu deux réunions en 2025 pour comprendre pourquoi les devis semblaient si raisonnables.

- **L'egress était une ligne « Skyvale ».** 9 700 EUR sans détail. Le rapport par hôte existait chez le fournisseur et personne ne l'ouvrait ([[egress-finding-map-tiles-2025-11]] est ce qu'on y a trouvé le jour où on l'a ouvert).

- **Pas de répartition par service.** La question « combien coûtent les notifications » avait pour réponse la facture Bipline, sans le calcul, sans Courrix, sans les pods. La première table par service ([[per-service-cost-table-q2-2026]] en descend) a pris trois semaines à construire parce qu'il fallait poser les étiquettes ([[cost-allocation-labels]]) avant de pouvoir compter.

- **Trimestriel.** Une facture anormale en janvier était vue en avril. Les 9 400 SMS expirés facturés en février 2026 auraient été payés sans qu'on le sache.

- **Une personne.** Quand elle était en congé, personne ne savait où était le fichier, et deux versions ont coexisté pendant un trimestre de 2024 avec des chiffres différents pour le même mois.

## Ce qui a été gardé

Les montants des factures de 2024 et 2025, recopiés dans `finops/costs.yaml` avec la mention `source: legacy-spreadsheet` et sans quantité, pour avoir une tendance sur deux ans même approximative. L'onglet du T3 2025 a servi de point de départ pour le premier rapport d'octobre 2025, et la première chose ajoutée a été la ligne d'amortissement, ce qui a fait « monter » le coût de 45 % en une réunion sans qu'un centime de plus soit dépensé.

## Ce que le remplacement a changé

Un fichier versionné, une quantité et une source par ligne, un rapport mensuel généré, une réconciliation contre l'usage, une répartition par service, une revue avec un ordre du jour ([[finops-monthly-review-feedback]]) et trois personnes qui tournent au lieu d'une. Le coût de l'infrastructure n'a pas changé le jour de la bascule ; ce qu'on en savait, oui.
