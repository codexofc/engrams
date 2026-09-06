---
name: tracker-vehicle-mapping
description: tracker_mappings links (provider, vehicle ref) to a vehicle_id with validity dates, auto-created from install events or plate matching, 4 % need manual fixes
type: reference
status: active
verified: 2026-03-24
---

# Mapping boîtier vers véhicule

Un fournisseur parle de `GLX-4471` ou de l'identifiant IMEI d'un boîtier ; la plateforme parle de `vehicles.id` (un camion d'un transporteur, avec son immatriculation). Entre les deux : `tracker_mappings`.

## La table

```
tracker_mappings
  id, provider ('trakko'|'geolyx'|'app'), provider_vehicle_ref,
  vehicle_id, carrier_organization_id,
  valid_from, valid_to (null = actif),
  source ('install_event'|'plate_match'|'manual'|'carrier_confirmed'),
  confirmed_at, confirmed_by
```

Contrainte : au plus un mapping actif par `(provider, provider_vehicle_ref)` (index unique partiel sur `valid_to IS NULL`). Un véhicule peut avoir plusieurs mappings actifs (un boîtier Trakko et l'application), voir [[position-source-priority]].

L'étape de mapping du pipeline ([[position-ingestion-pipeline]]) cherche le mapping actif à `device_ts`, pas à maintenant : un boîtier déplacé d'un camion à un autre le 10 mars donne des positions au premier camion jusqu'au 10 mars et au second ensuite, même si les positions arrivent en retard.

Pour l'application mobile, `provider_vehicle_ref` est l'identifiant de l'appareil et le mapping est créé quand le chauffeur choisit son véhicule au début de la mission (`assignment.started` porte `vehicle_id`), `valid_to` posé à la fin de la mission. Aucun geste du transporteur.

## Création automatique

**Trakko** : l'événement `device_installed` ([[trakko-api-contract-quirks]]) donne l'IMEI et l'immatriculation saisie par l'installateur. On cherche `vehicles.plate` normalisée (majuscules, sans espaces ni tirets) chez les transporteurs qui ont commandé des boîtiers. Une correspondance exacte crée le mapping avec `source = 'plate_match'`, en attente de confirmation. Pas de correspondance : ligne dans `unmapped_trackers`, et l'écran de mapping du transporteur la propose.

**Geolyx** : le premier lot d'un abonnement liste les `vehicle_ref` ; on appelle `GET /partners/vehicles/{ref}` pour obtenir l'immatriculation déclarée chez Geolyx et on fait la même correspondance.

**Taux** : 96 % des boîtiers trouvent leur véhicule automatiquement. Les 4 % restants : immatriculation mal saisie (`AB-123-CD` devenu `AB-123-CO`), véhicule pas encore créé chez nous, ou plaque étrangère avec un format qu'on normalise mal (les plaques polonaises avec espace variable ont demandé une règle dédiée).

## Confirmation par le transporteur

Un mapping `plate_match` fonctionne immédiatement (les positions passent) mais l'écran `/settings/telematics` du transporteur affiche « à confirmer » avec la dernière position sur une carte. Le planificateur confirme ou corrige. Non confirmé au bout de 14 jours : e-mail de rappel ; au bout de 30 jours, on garde le mapping et on arrête de demander. Taux de confirmation à 30 jours : 71 %. Taux de correction (le transporteur change le véhicule) : 2,5 %, presque toujours des jumeaux (deux camions identiques, plaques consécutives, l'installateur s'est trompé de cabine).

## Changement de véhicule

Un boîtier Trakko déplacé : l'événement `device_removed` ferme le mapping (`valid_to`), le `device_installed` suivant en ouvre un autre. Un véhicule Geolyx vendu : le transporteur le retire dans Geolyx, on reçoit un `vehicle_ref` inconnu pour le nouveau, cycle normal. Un transporteur qui vend un camion **avec** le boîtier Trakko à un autre transporteur, sans nous prévenir : le mapping reste actif et les positions du nouveau propriétaire arrivent chez l'ancien. Arrivé deux fois ; détecté parce que les positions étaient hors de toute fenêtre de mission et que le boîtier apparaissait dans le rapport « boîtier actif sans mission depuis 30 jours ». La règle : un boîtier sans mission depuis 45 jours passe en `valid_to = now()` et le transporteur est prévenu.

## Écran de support

`/backoffice/telematics/mappings?ref=...` montre l'historique complet d'un boîtier ou d'un véhicule. C'est le premier endroit à regarder quand un dispatcher dit « je vois mon camion à Gdańsk alors qu'il est à Lyon ». Neuf fois sur dix, c'est un mapping.
