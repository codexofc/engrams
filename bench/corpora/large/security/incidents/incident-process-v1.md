---
name: incident-process-v1
description: The 2024 to early 2026 incident process had no severity scale, no scribe and post-mortems in a shared doc without action tracking; replaced 2026-02
type: reference
status: archived
superseded_by: [[incident-process-severity-levels]]
verified: 2025-09-20
---

# Ancien processus d'incident (2024 à janvier 2026)

Remplacé par [[incident-process-severity-levels]]. Gardé pour lire les anciens post-mortems avec leur contexte.

## Ce que c'était

- Un canal unique `#incidents` pour tout, disponibilité et sécurité mêlés. Les incidents sécurité y étaient noyés parmi les alertes de latence.

- Pas d'échelle de sévérité. Un incident était « un incident ». En pratique, la personne qui déclarait choisissait qui prévenir, et le DPO n'a été prévenu d'aucun incident en 2025 avant octobre.

- Pas de rôle de scribe. La chronologie était reconstruite après coup à partir de l'historique du chat, avec des trous : le post-mortem du phishing d'octobre 2025 a une heure d'incertitude sur le moment du premier clic parce que la personne qui suivait l'incident réparait en même temps.

- Le post-mortem était un document partagé, gabarit libre, avec une section « actions » sous forme de liste à cocher. En septembre 2025 on a compté : 31 actions listées depuis 2024, 12 cochées, 9 sans propriétaire, et pour les 10 restantes personne ne savait si elles étaient faites.

- Pas de critère de clôture. Un incident se terminait quand le canal se taisait.

## Ce qui a précipité le changement

Trois incidents entre octobre et décembre 2025 (phishing, secrets dans un log de build, et le début du chantier sur les URL présignées) traités par trois commandants différents avec trois façons de faire, et une revue de fin d'année où on n'a pas pu dire combien d'actions de post-mortem étaient closes.

Le nouveau processus a repris ce qui marchait (la déclaration facile, l'absence de blâme, qui était déjà la culture) et a ajouté ce qui manquait : sévérités, scribe, tickets pour les actions, critère de clôture, lien avec la notification conformité.
