---
name: flutter-upgrade-3-27
description: Driver app moved from Flutter 3.22 to 3.27 in HF-1480 (Feb 2026), Impeller became default on Android and broke the map overlay until the plugin update, startup time improved 18 %
type: project
status: active
verified: 2026-03-06
---

# Flutter 3.22 → 3.27 (HF-1480)

Livré dans l'app 4.8 le 2026-02-18. Deux sprints, une personne à mi-temps.

## Pourquoi maintenant

3.22 arrivait en fin de support des plugins qu'on utilise (le plugin de caméra exigeait 3.24 minimum pour un correctif de fuite mémoire sur Android 14 qu'on subissait). Et l'attente de Dart 3.6 pour les `digit separators` était accessoire.

## Ce qui a cassé

- **Impeller par défaut sur Android.** Le rendu de la carte (MapLibre via `PlatformView`) affichait un rectangle noir par-dessus les marqueurs sur les Galaxy A2x. Correctif : mise à jour du plugin de carte vers la version qui supporte le mode `hybrid composition` avec Impeller, plus `android:hardwareAccelerated="true"` qui manquait dans le manifest d'un flavor. Trois jours perdus.

- **`WillPopScope` déprécié** au profit de `PopScope`. 14 écrans. Le comportement du bouton retour sur l'écran de livraison en cours (qui demande confirmation) a régressé une fois parce que `PopScope.canPop` est évalué avant l'appui, pas après. Test d'intégration ajouté.

- **`TextTheme`** : les noms de styles (`bodyText1` etc.) étaient déjà migrés, rien à faire.

- **drift** : passage à drift 2.24 dans la foulée, migration de schéma sans changement, voir [[local-db-drift-schema-migrations]].

- **Gradle** : AGP 8.7 requis, `compileSdk 35`, ce qui a obligé à déclarer `FOREGROUND_SERVICE_LOCATION` explicitement et à justifier le type de service au premier plan dans le manifest. Le Play Store a demandé une vidéo de démonstration du suivi pour valider la permission, fournie, accepté en 4 jours.

## Ce qui s'est amélioré

- Démarrage à froid sur Galaxy A25 : 2,8 s → 2,3 s (mesuré avec `flutter run --trace-startup`, moyenne de 10 lancements). 18 %.

- Taille de l'APK universel : 61 Mo → 58 Mo. Voir [[app-size-budget]].

- Défilement de la liste des chargements : plus de saccades à 60 images par seconde sur l'A25 (il y en avait avec Skia sur les listes avec photos).

## Ce qui n'a pas changé

Pas de passage à `flutter_riverpod` 3 en même temps, un upgrade à la fois. Pas de Swift Package Manager côté iOS, on reste sur CocoaPods jusqu'à ce que tous les plugins suivent.

## Leçon

Tester la carte sur un vrai appareil bas de gamme dès le premier jour d'un upgrade de Flutter. L'émulateur avec Impeller ne montrait rien d'anormal. La liste des appareils de test est dans [[ci-pipeline-mobile-runners]].
