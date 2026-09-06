---
name: eta-features
description: The 47 ETA features by group with their source, and the six most important including remaining_driving_minutes and borders
type: reference
status: active
verified: 2026-06-17
---

Liste des features de `eta-v3` ([[eta-model-overview]]), avec leur source. Les noms sont ceux du feature store ([[feature-store-design]]), groupe `eta`.

## Itinéraire (9)

`route_driving_minutes` (moteur de routage, calculé à la publication), `route_distance_km`, `route_motorway_share`, `border_crossings` (nombre de frontières, y compris intra-Schengen : les contrôles sont rares mais les zones frontalières sont lentes), `ferry_minutes`, `route_countries` (encodé), `mountain_pass_count` (cols et tunnels alpins listés à la main, 14 entrées), `toll_free_share`, `urban_end_share` (part des 20 derniers km en zone urbaine dense).

## État du conducteur (8)

`remaining_driving_minutes` (temps de conduite restant avant repos obligatoire, remonté par l'app conducteur toutes les 10 minutes), `remaining_daily_minutes`, `hours_since_rest`, `is_double_crew`, `rest_due_within_route` (booléen dérivé : le repos tombera pendant le trajet), `driver_loads_last_7d`, `driver_avg_speed_last_3_loads`, `driver_app_version_band`.

`remaining_driving_minutes` est la feature la plus importante du modèle après le temps de routage : un conducteur à 40 minutes de son repos obligatoire sur un trajet de 3 heures arrivera 9 à 11 heures plus tard, et aucun modèle sans cette information ne peut le voir. La détection des repos effectifs à partir des positions est décrite dans [[rest-stop-detection]].

## Temps (7)

`departure_hour_local`, `departure_dow`, `is_holiday_origin`, `is_holiday_destination`, `is_holiday_transit` (un jour férié dans un pays traversé, voir [[holiday-calendar-feature]]), `weekend_driving_ban_hours` (heures d'interdiction de circuler pour les poids lourds sur le trajet, calculées à partir du calendrier des interdictions par pays), `week_of_year`.

## Historique transporteur (7)

`carrier_lateness_p50_90d` (retard médian à la livraison des 90 derniers jours), `carrier_lateness_p90_90d`, `carrier_loads_90d`, `carrier_fleet_band`, `carrier_no_show_rate_180d`, `carrier_lane_familiarity` (chargements du transporteur sur ce cluster de ligne dans l'année), `carrier_country`.

## Chargement (8)

`vehicle_type`, `adr_class`, `is_temperature_controlled`, `stop_count`, `declared_weight_band`, `pickup_window_minutes`, `delivery_window_minutes`, `shipper_site_avg_loading_minutes` (temps moyen passé sur le site du chargeur, calculé à partir des positions, très discriminant pour les sites industriels à 3 heures d'attente).

## Position courante (8, nulles au contexte `at_award`)

`remaining_route_minutes` (réestimé depuis la position), `remaining_distance_km`, `progress_ratio`, `minutes_since_departure`, `speed_last_15min`, `stopped_minutes_last_60`, `deviation_from_route_km`, `position_age_minutes`.

## Importance (réentraînement du 2026-06-15, gain total normalisé)

1. `route_driving_minutes` 0,31
2. `remaining_driving_minutes` 0,14
3. `remaining_route_minutes` 0,11
4. `border_crossings` 0,06
5. `shipper_site_avg_loading_minutes` 0,05
6. `carrier_lateness_p50_90d` 0,04

Le reste sous 0,03 chacun. Les features de temps pèsent peu individuellement mais leur suppression en bloc coûte 4 minutes de MAE.

## Ce qui n'y est pas

- La météo : testée en 2025, gain de 40 secondes de MAE, abandonnée.
- Le prix de l'enchère : un transporteur mal payé n'est pas plus en retard, mesuré, corrélation nulle.
- L'identité du conducteur : le transporteur oui, le conducteur non, par choix (voir [[ml-team-preferences]]).
