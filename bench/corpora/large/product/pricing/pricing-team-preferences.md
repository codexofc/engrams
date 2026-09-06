---
name: pricing-team-preferences
description: Pricing team working preferences, decisions by written experiment not by opinion, percentages on base price, config tables versioned, Python for pricing-svc, and no pricing change deployed on a Friday or before a fuel index publication
type: user
status: active
verified: 2026-03-27
---

Préférences de l'équipe pricing (deux développeurs, un analyste, un product manager).

- **Une décision de prix se prend sur une expérience écrite**, jamais sur une intuition en réunion, même quand l'intuition est bonne. Le gabarit est dans [[pricing-experiment-guidelines]]. Une exception : une correction de bug de calcul, qui se déploie sans expérience mais avec le rejeu de devis réels.
- **Les pourcentages s'appliquent sur le prix de base**, additionnés, jamais en cascade. Voir [[weekend-surcharge-backfire]] pour la raison. Quiconque propose « mais logiquement le carburant devrait s'appliquer sur le prix majoré » relit la note.
- **Toute table de configuration est versionnée** avec `valid_from` et `valid_to`, et un devis garde sa `config_version`. On ne fait pas d'`UPDATE` sur une ligne de tarif.
- **pricing-svc reste en Python**, bid-svc en Kotlin. On a discuté de réécrire pricing-svc en Kotlin pour n'avoir qu'un langage ; refusé, le modèle et l'analyse sont en Python et l'équipe préfère un service de plus qu'une frontière langage au milieu du calcul.
- **Pas de déploiement de pricing-svc le vendredi après 14:00**, ni le 2 et le 3 du mois (publication de l'indice carburant, voir [[fuel-surcharge-index]]). Le week-end est le moment où les majorations se combinent le plus et où personne ne regarde.
- **Les prix dans le code sont en cents entiers**, les pourcentages en points de base entiers (1 200 pour 12 %). Un `float` dans un montant est refusé en revue.
- **L'analyste a accès en lecture à l'entrepôt**, pas à la base de production. Toute analyse part de l'entrepôt ; si l'entrepôt n'a pas la donnée, on l'y ajoute plutôt que d'interroger la production.
- **Les réunions** : revue des métriques le mardi 11:00 (30 minutes, tableau de bord à l'écran, pas de slides), et une revue d'expérience quand une expérience se termine.
