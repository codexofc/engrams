---
name: geofence-arrival-detection
description: Arrival is set when 2 consecutive positions fall within the site radius (150 m default), departure after 3 outside for 5 min; auto-sets 81 % of stops
type: project
status: active
verified: 2026-05-08
---

# Détection d'arrivée par géorepérage (HF-2100)

Avant, l'heure d'arrivée à un site était celle où le chauffeur appuyait sur « Arrivé ». En moyenne 11 minutes après l'arrivée réelle, parfois le lendemain. La facturation des attentes et les litiges « le camion était en retard » reposaient sur ce bouton. Le consommateur `geofence-consumer` lit le topic `positions` ([[position-ingestion-pipeline]]) et pose l'heure lui-même.

## La règle

Pour chaque position conservée d'un véhicule en mission :

- **Arrivée** : la position est à moins de `radius_m` du point d'arrêt (enlèvement ou livraison) **et** la précédente aussi. Deux positions consécutives, pour éviter qu'un camion qui passe devant sur l'autoroute compte comme arrivé. `stops.arrived_at` est posée à l'heure de la **première** des deux positions, événement `stop.arrival_detected`, source `geofence`.

- **Départ** : trois positions consécutives hors du rayon sur au moins 5 minutes. `stops.departed_at` à l'heure de la première position hors rayon. Le délai de 5 minutes évite qu'une manœuvre dans une grande plateforme logistique (où le rayon peut être dépassé) compte comme un départ ; c'est la raison de la révision de la déduplication à 5 m ([[position-dedup-rules]]).

- **Rayon** : 150 m par défaut. Par site (`sites.geofence_radius_m`) quand le chargeur ou le transporteur l'ajuste : 400 m pour les grandes plateformes, 60 m pour une adresse en centre-ville où deux clients se partagent la rue. 210 sites ont un rayon personnalisé en mai 2026, la plupart posés par le support après un ticket.

Le chauffeur voit dans l'application « Arrivée détectée à 09:42, confirmer ? » avec un bouton qui confirme et un autre qui corrige l'heure (à plus ou moins 30 minutes, au-delà c'est le dispatcher). La confirmation n'est pas obligatoire ; l'heure détectée reste si personne ne fait rien.

## Résultats

Sur avril 2026 : 81 % des arrêts ont une arrivée détectée automatiquement ; 14 % ont une arrivée par bouton seulement (véhicule sans source de position à ce moment, ou zone blanche) ; 5 % ont eu une correction du chauffeur, médiane de la correction 6 minutes, presque toujours vers plus tôt (le camion attendait à la barrière, hors rayon).

Écart médian entre l'arrivée détectée et l'arrivée confirmée par le chauffeur quand les deux existent : 2 minutes. L'écart médian entre l'ancien bouton seul et l'arrivée détectée, sur les arrêts où on a les deux : 11 minutes, ce qui confirme le problème de départ.

## Cas qui ont demandé une règle

- **Sites sans coordonnées fiables** : le géocodage de l'adresse donne le centre de la commune. Si la distance entre l'adresse géocodée et la première arrivée détectée pour ce site dépasse 500 m, on ne détecte plus rien pour ce site et on l'envoie dans la file « à géolocaliser » du support, qui pose le point sur la carte. 340 sites corrigés ainsi depuis janvier.

- **Deux arrêts au même site** (enlèvement et livraison chez le même client, ou deux livraisons dans la même plateforme) : la détection s'applique à l'arrêt suivant non encore arrivé dans l'ordre de la mission, jamais à deux arrêts à la fois.

- **Position ancienne** : si `position_age_s` de la position qui déclenche dépasse 300 s (rattrapage, rejeu), on pose l'arrivée avec `source = 'geofence_late'` et on n'envoie pas la notification temps réel au chargeur, seulement l'événement.

- **Chauffeur en opposition au suivi** : pas de position, pas de détection, le bouton reste le seul moyen, voir [[telematics-consent-and-masking]].

## Ce qu'on n'a pas fait

- Une détection « en approche » (à 5 km) pour prévenir le site. Demandé par deux chargeurs, refusé pour l'instant : l'ETA ([[eta-feed-publication]]) donne déjà l'information et une notification de plus toutes les 5 minutes est exactement ce qu'on a réduit.

- Utiliser l'état d'allumage (Trakko, Geolyx) comme signal d'arrêt. Trop de camions restent allumés à quai (frigo, chauffage), mesuré à 40 % des arrêts.

Facturation des attentes : `stops.arrived_at` et `departed_at` avec `source` sont ce que la facturation lit ; un arrêt dont l'attente dépasse le seuil contractuel avec `source = 'geofence'` est facturé sans intervention, avec `source = 'driver'` il passe en revue. C'est la raison pour laquelle la précision de la détection a eu un budget.
