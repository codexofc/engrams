---
name: app-size-budget
description: Keep the universal Android APK under 60 MB and the iOS download under 45 MB, check the delta in CI, prefer removing a dependency over adding one, and the list of what the size is made of
type: feedback
status: active
verified: 2026-03-13
---

# Budget de taille de l'app

Règle : **APK universel Android sous 60 Mo, téléchargement iOS sous 45 Mo.** Le CI compare la taille de l'APK release avec `main` et bloque à +1 Mo sans le label `size-ok` (voir [[ci-pipeline-mobile-runners]]).

## Pourquoi ça compte pour nous

- Les chauffeurs sans services Google téléchargent l'APK en direct sur données mobiles (voir [[device-quirk-huawei-no-gms]]). 60 Mo, c'est déjà long.

- Les téléphones bas de gamme ont 32 Go dont 10 libres. Une app de 150 Mo installée est la première qu'on désinstalle.

- Chaque Mo ajouté à l'app est là pour toujours, chaque Mo ajouté au réseau est temporaire. Quand on hésite, on télécharge à la demande.

## De quoi c'est fait (app 4.9, APK arm64 seul : 38 Mo, universel : 58 Mo)

- Moteur Flutter et code Dart compilé : 14 Mo. Incompressible.

- Bibliothèque de carte (MapLibre, natif) : 9 Mo. Le plus gros poste évitable, mais la carte est centrale.

- Polices : 2,1 Mo. On embarque une seule famille en 3 graisses, avec sous-ensemble latin étendu et cyrillique (chauffeurs bulgares). Le sous-ensemble a fait gagner 4 Mo par rapport à la police complète.

- Images et icônes : 1,4 Mo, tout en SVG ou WebP. Un PNG de 800 Ko pour l'écran de bienvenue a été remplacé par un SVG de 12 Ko en 2025.

- Plugins natifs (caméra, sécurité du stockage, notifications, scanner) : 6 Mo.

- Traductions (6 langues, ARB compilés) : 300 Ko.

## Comment appliquer

- Avant d'ajouter une dépendance, mesurer : `flutter build apk --release --analyze-size --target-platform android-arm64` et comparer. Une bibliothèque de détection de bords de document a été refusée à +9 Mo, voir [[pod-photo-compression]].

- Préférer un package Dart pur à un plugin natif quand les deux existent, le natif embarque souvent une bibliothèque complète pour une fonction.

- Le `--split-per-abi` réduit ce que le store livre (l'App Bundle fait ça tout seul), mais la cible reste l'APK universel parce que c'est ce que téléchargent les appareils sans store.

- Retirer ce qui ne sert plus : l'audit trimestriel de `pubspec.yaml` a enlevé 5 packages en 2025 (dont un client HTTP en doublon).

## Ce qu'on n'a pas fait

Le chargement différé de code Dart (`deferred as`) fonctionne sur Android via les modules dynamiques du Play Store, pas sur les APK directs ni sur iOS. Pas rentable pour nous.
