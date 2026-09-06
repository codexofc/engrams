---
name: search-geo-radius-queries
description: Recherche par rayon dans haystack : point de recherche et ses défauts, rayons jusqu'à 800 km, zone de livraison, géocodage, chargements sans coordonnées
type: reference
status: active
verified: 2026-06-05
---

# Requêtes géographiques dans la recherche transporteur

Un transporteur cherche « ce qui part près d'ici vers là-bas ». « Près d'ici » est un rayon autour d'un point ; « vers là-bas » est une zone. Les deux sont des filtres ([[search-relevance-rules-v2]]) ; la distance au point sert aussi au classement.

## Le point de recherche

Trois origines possibles, dans cet ordre de priorité :

1. Un point choisi explicitement (une ville tapée, géocodée côté API, ou un point posé sur la carte).

2. La position GPS du téléphone si le transporteur l'a autorisée (app 4.10, « Utiliser ma position »).

3. Le dernier point de recherche du transporteur, mémorisé par utilisateur.

4. À défaut, l'adresse enregistrée de l'organisation. C'était le défaut unique avant août 2026 et la cause du retour Ravello côté support : pour un artisan, l'adresse enregistrée est souvent celle du comptable.

Le point est envoyé à l'API en `lat,lon` avec quatre décimales (une dizaine de mètres, largement assez).

## Le rayon

Défaut 150 km, choix parmi 50, 100, 150, 200, 300, 500, 800. Pas de rayon libre, pas au-delà de 800 km : au-delà, c'est une recherche par pays, qu'on propose à la place (filtre `pickup.country`). Le rayon est un filtre `geo_distance` sur `pickup.geo` :

```json
{ "geo_distance": { "distance": "150km", "pickup.geo": { "lat": 45.44, "lon": 10.99 } } }
```

La distance est une distance à vol d'oiseau (arc). Le transporteur voit la distance routière estimée dans la carte de résultat (`distance_km` de son point au pickup, calculée par l'API à l'affichage avec la table de distances du pricing), ce qui fait qu'un chargement « à 148 km » dans le filtre peut afficher « 190 km par la route ». Documenté dans l'aide ; deux tickets en 2026.

## La zone de livraison

Optionnelle. Trois formes :

- un rayon autour d'un second point (« vers Lyon, 100 km »), même filtre sur `delivery.geo` ;

- un ou plusieurs pays (`delivery.country`) ;

- un polygone dessiné sur la carte (web uniquement), envoyé en GeoJSON, filtre `geo_polygon` limité à 50 sommets. Peu utilisé (2 % des recherches) mais les transporteurs qui l'utilisent y tiennent : ce sont ceux qui rentrent chez eux le vendredi.

## Géocodage des chargements

`pickup.geo` et `delivery.geo` viennent de `loads.pickup_geo` et `loads.delivery_geo`, remplies par le géocodeur de l'API à la création du chargement à partir de l'adresse (code postal, ville, pays ; la rue affine si elle est reconnue). Le géocodeur est un service interne alimenté par une base d'adresses ouverte, avec un cache Redis par adresse normalisée ; taux de succès 99,1 % sur 2026.

Un chargement sans coordonnées (0,9 %) est indexé avec `pickup.geo` absent : il n'apparaît dans **aucune** recherche par rayon, seulement dans les recherches par pays et par ville exacte (`pickup.city.raw`). Le chargeur voit un avertissement sur sa fiche (« adresse non localisée, visibilité réduite ») depuis HF-3195 et corrige en général dans l'heure. Le playbook du support en parle.

## Précision et cas limites

- Deux chargements au même endroit (une plateforme logistique) ont le même `pickup.geo` ; le classement les départage par les autres composantes.

- Les points près d'une frontière : un transporteur à Strasbourg avec 100 km voit Karlsruhe ; c'est voulu, la recherche ignore les pays sauf filtre explicite.

- L'antiméridien et les pôles : non gérés, on ne livre pas là.

- Un point de recherche en mer (un transporteur qui clique au hasard sur la carte) : accepté, il verra les chargements côtiers.

## Coût

Le filtre `geo_distance` sur un `geo_point` indexé en BKD est bon marché : 2 à 4 ms sur 60 000 documents, quel que soit le rayon. Ce qui coûte, c'est le `gauss` du classement sur les candidats restants, proportionnel à leur nombre, d'où l'intérêt de garder le filtre de dates et de véhicule serré avant le score. Voir [[search-query-latency-slo]] pour les chiffres.
