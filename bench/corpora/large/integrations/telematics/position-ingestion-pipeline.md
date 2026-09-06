---
name: position-ingestion-pipeline
description: Positions flow adapter, mapping, window filter, plausibility, dedup, COPY into position_events, then Kafka; p95 under 12 s pushed, 45 s polled; stage counters
type: reference
status: active
verified: 2026-05-30
---

# Pipeline d'ingestion des positions

Le chemin d'une position dans `hf-telematics-gw`, du fournisseur à la table et au topic. La version précédente (un cron par fournisseur qui écrivait directement en base) est archivée dans [[position-ingestion-v1-polling]].

## Vue d'ensemble

```
adapter (Trakko poll | Geolyx webhook | app upload)
  -> RawPosition
  -> MappingStage        (provider vehicle id -> vehicle_id, carrier_organization_id)
  -> WindowStage         (dans une fenêtre de mission ? sinon jeté)
  -> PlausibilityStage   (fix nul, saut impossible, horodatage aberrant)
  -> DedupStage          (voir position-dedup-rules)
  -> PersistStage        (COPY vers position_events par lots de 2 000)
  -> PublishStage        (topic Kafka positions, clé = vehicle_id)
```

Chaque étape est une classe `Stage` avec `process(iterable<RawPosition>): iterable<RawPosition>` et un compteur Prometheus `telematics_stage_total{stage, provider, outcome}`. Le tableau de bord montre les six barres par fournisseur, et la première question sur « il manque des positions » est « à quelle étape ».

## Les étapes

**Adapter.** Produit des `RawPosition` avec `provider`, `provider_vehicle_ref`, `device_ts`, `received_ts`, `lat`, `lon`, `heading`, `speed`, `accuracy_m`, `ignition`, `odometer_m`, `raw_ref` (clé de l'objet brut dans `hf-telematics-raw`, gardé 7 jours pour le débogage). Trakko : un poller par flotte, 30 s. Geolyx : contrôleur webhook, file interne, 4 handlers ([[geolyx-push-webhook-format]]). Application : endpoint `POST /v2/positions/batch` de l'API, qui délègue au gateway par le mesh.

**MappingStage.** `provider_vehicle_ref` vers `vehicle_id` et `carrier_organization_id` via `tracker_mappings` ([[tracker-vehicle-mapping]]), cache mémoire 60 s. Non mappé : compté `outcome=unmapped`, écrit dans `unmapped_positions` (dernière position par ref, pour l'écran de mapping), pas plus loin.

**WindowStage.** La règle de conformité : une position n'est conservée que si le véhicule a une mission active à `device_ts`, plus 30 minutes avant le créneau d'enlèvement. `AssignmentWindowIndex` tient en mémoire les fenêtres des 48 prochaines heures et des 2 dernières, rafraîchies toutes les 60 s depuis la plateforme. Hors fenêtre : `outcome=outside_window`, jeté, pas stocké, pas même en brut. Détails et cas limites dans [[telematics-consent-and-masking]].

**PlausibilityStage.** Jette : `lat = lon = 0` ; `accuracy_m > 500` ; un saut de plus de 200 km/h par rapport à la dernière position connue du véhicule (cache Redis `lastpos:{vehicle}`) ; `device_ts` dans le futur de plus de 60 s ou dans le passé de plus de 24 h (les rattrapages Trakko passent par un chemin séparé qui lève cette limite). La vitesse est utilisée ici puis **retirée** de l'enregistrement : `position_events` n'a pas de colonne vitesse, décision conformité.

**DedupStage.** [[position-dedup-rules]].

**PersistStage.** Lots de 2 000 lignes ou 2 secondes, `COPY position_events FROM STDIN` sur la partition du jour ([[positions-table-partitioning]]). Une transaction par lot. Échec : le lot revient en file, rejoué ; la clé d'idempotence est `(vehicle_id, ts, provider)` en index unique sur la partition, donc un rejeu ne duplique pas.

**PublishStage.** Message Kafka sur `positions`, clé `vehicle_id` (donc ordre par véhicule), valeur JSON compacte sans `raw_ref`. Consommateurs : ETA ([[eta-feed-publication]]), géorepérage ([[geofence-arrival-detection]]), et le WebSocket de la carte des dispatchers via la plateforme. Publié **après** persistance ; un consommateur qui a vu une position la retrouve en base.

## Latence

Mesurée de `received_ts` à l'écriture Kafka, mai 2026 : p50 3 s, p95 11 s pour Geolyx et l'application ; p95 44 s pour Trakko, dont 30 s de période de sondage. L'objectif écrit est p95 sous 15 s pour les sources poussées et sous 60 s pour les sources sondées.

## Débit et dimensionnement

41 millions de bruts par jour, 9,2 millions écrits. Pic observé : 2 400 bruts par seconde pendant 20 minutes ([[incident-2025-11-geolyx-duplicate-flood]]). Le gateway tourne sur 3 pods, chacun tient 1 500 par seconde en charge de test ([[telematics-integration-test-harness]]) ; la marge est là pour un rattrapage, pas pour la croissance, qu'on réévalue chaque trimestre.

## Ce qu'on ne fait pas dans le pipeline

- Pas de calcul d'ETA ni de détection d'arrêt ici : ce sont des consommateurs du topic.

- Pas d'enrichissement (adresse, pays) : la carte fait du géocodage inverse à l'affichage, l'entrepôt de données le fait en lot.

- Pas de stockage du brut au-delà de 7 jours, et pas du tout pour les positions hors fenêtre.

## Déboguer « il manque des positions »

Dans l'ordre, parce que c'est l'ordre qui va le plus vite :

1. **Tableau de bord des étapes**, filtré sur le fournisseur : quelle barre a chuté et quand. Une chute de `adapter` est un problème fournisseur ou réseau ; de `mapping`, un boîtier déplacé ou un nouveau véhicule ; de `window`, une mission qui n'a pas démarré ; de `plausibility`, un boîtier qui déraille ; de `persist`, la base.

2. **`telematics:trace --vehicle <id> --since 1h`** : rejoue le chemin des positions brutes de ce véhicule encore dans `hf-telematics-raw` à travers les étapes, en mode lecture, et affiche pour chacune l'issue et la raison. C'est la commande qui répond à « pourquoi celle-là a été jetée ».

3. **`unmapped_positions`** si le véhicule est absent : le boîtier envoie mais n'est mappé à rien.

4. **Le cache des fenêtres** : `telematics:windows --vehicle <id>` montre les fenêtres de mission chargées en mémoire pour ce véhicule ; si la mission existe sur la plateforme mais pas ici, le rafraîchissement de 60 s est en retard ou a échoué (compteur `window_refresh_errors_total`).

Neuf tickets sur dix se résolvent à l'étape 1 ou 3. Le dixième est en général un horodatage, et la note sur la dérive des horloges Trakko explique quoi regarder.

## Ce qui a changé depuis la mise en service

- Janvier 2026 : file Redis Streams remplacée par Kafka entre les adaptateurs et les étapes, après la rafale Geolyx ; la rétention du topic brut est de 24 h, ce qui donne une deuxième chance de rejouer sans le bucket.

- Février 2026 : règle de l'écart d'horodatage dans l'adaptateur Trakko.

- Mars 2026 : `positions_selected` ajouté en sortie, pour la priorité entre sources.

- Mai 2026 : lots de `COPY` passés de 1 000 à 2 000 lignes après mesure ; au-delà, la latence de la dernière position du lot dépassait l'objectif sans gain de débit.
