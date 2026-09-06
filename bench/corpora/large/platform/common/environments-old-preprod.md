---
name: environments-old-preprod
description: Historical four-tier layout with a preprod environment on a separate cluster, removed in 2025 because it was never in sync with production and cost a third of the infra budget
type: reference
status: archived
superseded_by: [[environments-dev-staging-prod]]
verified: 2025-09-12
---

# Ancien découpage avec preprod (jusqu'en 2025)

Jusqu'à l'été 2025 il y avait quatre environnements : `dev`, `staging`, `preprod` et `prod`. `preprod` tournait sur un cluster séparé, avec une copie de la base de production non anonymisée (ce qui posait un problème que la sécurité a fini par refuser), et servait aux tests de charge et aux répétitions de migration.

Ce qui a mené à sa suppression :

- Il n'était jamais à jour : la copie de base datait de 2 à 6 semaines, les versions déployées étaient celles de la dernière répétition. Les répétitions de migration s'y faisaient contre un schéma qui n'était pas celui de la production.

- Un tiers du budget infrastructure pour un cluster utilisé quelques heures par mois.

- Les tests de charge y donnaient des résultats non comparables à la production (stockage différent, pas de PgBouncer).

Remplacé par : le staging restauré chaque nuit et anonymisé, et les tests de charge faits en production sur un sous-ensemble de trafic avec un flag. Voir [[environments-dev-staging-prod]].
