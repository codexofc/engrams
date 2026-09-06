---
name: holiday-calendar-feature
description: Holiday and truck driving-ban features from country_calendar and driving_bans.yaml, per origin, destination and transit country
type: project
status: active
verified: 2026-01-19
---

## Deux sources

1. **Jours fériés** : le dictionnaire `country_calendar` de l'entrepôt (maintenu par l'équipe data, un fichier CSV versionné, jours fériés nationaux, union des Länder pour l'Allemagne). L'équipe ML ne tient pas son propre calendrier ; c'est la seule source autorisée, et les deux équipes l'ont écrit chacune dans leurs notes pour que ça tienne.

2. **Interdictions de circuler** pour les poids lourds : `driving_bans.yaml` dans le dépôt `ml-features`, maintenu par l'équipe ML parce que personne d'autre n'en a besoin. Par pays : plages hebdomadaires (France : dimanche 00:00 à 22:00 et samedi 22:00 à dimanche 00:00 ; Allemagne : dimanche 00:00 à 22:00 ; Pologne : samedi 18:00 à dimanche 22:00 en été seulement ; Autriche : samedi 15:00 à dimanche 22:00 ; Italie : dimanche 08:00 à 22:00 avec des horaires d'été différents), plus les interdictions des jours fériés et les interdictions estivales (les samedis de juillet et août en France, les « Ferienreiseverordnung » allemandes).

## Features dérivées

Pour l'ETA ([[eta-features]]) :

- `is_holiday_origin`, `is_holiday_destination`, `is_holiday_transit` : booléens sur la date de départ prévue.
- `weekend_driving_ban_hours` : nombre d'heures d'interdiction que le trajet rencontre, calculé en simulant le trajet heure par heure sur l'itinéraire (pays traversés avec leurs heures d'entrée estimées) contre `driving_bans.yaml`. C'est un calcul de 2 ms à la publication du chargement, stocké dans `feat.eta_load`.

Pour la demande ([[demand-forecast-model]]) : `is_holiday`, `is_day_before_holiday`, `is_day_after_holiday`, `holiday_in_week` (nombre de fériés dans la semaine), par pays d'origine et de destination du cluster.

## Effets mesurés

- L'interdiction du dimanche en France et en Allemagne est le plus gros effet calendaire de l'ETA : un chargement parti le samedi 14:00 de Varsovie pour Lyon arrive le lundi matin, pas le dimanche, et sans cette feature le modèle v1 se trompait de 14 heures en médiane sur ces trajets.
- Pour la demande, le jour d'après férié (« lendemain ») a réduit le biais des semaines fériées de 5 % à 2 %.
- Les fériés régionaux non couverts (Corpus Christi en Bavière et non en Bade-Wurtemberg, par exemple) coûtent environ 3 minutes de MAE sur les chargements concernés, accepté.

## Maintenance

- `country_calendar` : mis à jour chaque octobre pour l'année suivante par l'équipe data.
- `driving_bans.yaml` : revu chaque janvier (les gouvernements publient les dérogations estivales entre mars et mai, deuxième passage en mai). Un test dans `ml-features` vérifie que chaque pays traversé par au moins 1 % des chargements a une entrée, et échoue sinon ; la Slovaquie a été ajoutée comme ça en 2025.
- Une dérogation ponctuelle (levée d'interdiction pour une grève, une pénurie) n'est pas prise en compte ; trop rare, et l'information arrive trop tard.
