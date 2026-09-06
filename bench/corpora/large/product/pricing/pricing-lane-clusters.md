---
name: pricing-lane-clusters
description: 2 140 lane clusters at NUTS-3 with distance band and vehicle type, confidence from 90-day awarded loads, base price is the median per km
type: project
status: active
verified: 2026-05-22
---

Le prix de base d'un devis vient du cluster de ligne du chargement. Un cluster est une paire (zone d'origine, zone de destination) au niveau NUTS-3 (département en France, Kreis en Allemagne, powiat en Pologne), croisée avec une bande de distance (moins de 150 km, 150 à 400, 400 à 800, plus de 800) et un type de véhicule (tautliner, frigo, méga, porteur). Table `lane_clusters`, 2 140 clusters actifs en mai 2026.

## Prix de base

Pour chaque cluster, `pricing-svc` recalcule chaque nuit :

- `median_eur_per_km` : médiane du prix attribué par kilomètre sur les chargements attribués des 90 derniers jours, hors majorations (on retire `toll`, `ferry`, `adr`, `temperature` des enchères pour retrouver un prix de base comparable, ce qui suppose que la décomposition stockée avec l'enchère soit correcte, voir [[surcharge-rules-catalog]]) ;
- `n_awarded_90d` : nombre de chargements attribués sur la fenêtre ;
- `confidence` : `high` si `n_awarded_90d >= 40`, `medium` entre 12 et 39, `low` en dessous.

Le prix de base d'un devis est `median_eur_per_km * distance_km`, avec un plancher de 1,05 EUR/km sur les distances de moins de 150 km (les trajets courts ont des coûts fixes que la médiane par km sous-estime) et un plancher absolu de 180 EUR.

## Clusters à faible confiance

31 % des clusters sont `low`, mais ils ne portent que 6 % des chargements. Pour eux, le prix de base est un mélange : 40 % de la médiane du cluster (quand elle existe), 60 % de la prédiction du modèle de suggestion ([[price-suggestion-model-v2]]) qui généralise à partir des clusters voisins. Le devis porte `confidence`, et l'ancrage dans le formulaire n'est activé qu'en `medium` ou `high` (décision de [[experiment-anchor-price-2026-03]]).

## Pourquoi NUTS-3 et pas les codes postaux

On a essayé les deux premiers chiffres du code postal en 2024 : ça donne 95 zones en France mais 100 zones en Allemagne très hétérogènes en taille, et en Pologne les deux premiers chiffres couvrent parfois une demi-voïvodie. NUTS-3 est comparable d'un pays à l'autre et les données de l'office statistique européen sont libres. La correspondance code postal vers NUTS-3 est dans la table `postal_nuts3`, 240 000 lignes, rechargée deux fois par an.

## Ce qui reste faux

- Les clusters retour (backhaul) : un transporteur qui rentre à vide accepte moins que la médiane, et la médiane du cluster inverse ne le reflète pas. On a une colonne `imbalance_ratio` (chargements A vers B sur B vers A) qui sert de feature au modèle mais pas encore de correction directe du prix de base.
- Les bandes de distance sont trop larges au-dessus de 800 km : un Lisbonne-Varsovie et un Lyon-Berlin sont dans la même bande. On envisage 800 à 1 300 et plus de 1 300 au T3 2026, ticket HF-2590.
