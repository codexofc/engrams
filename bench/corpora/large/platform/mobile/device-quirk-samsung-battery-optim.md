---
name: device-quirk-samsung-battery-optim
description: Samsung One UI puts the app to sleep after 3 days unused and kills the tracking foreground service under Adaptive Battery, mitigated by the exemption request screen and the in-app check on Settings.canDrawOverlays-like signals
type: project
status: active
verified: 2026-02-18
---

# Samsung : mise en veille de l'app et suivi GPS interrompu

Environ 55 % de nos chauffeurs Android sont sur Samsung (télémétrie de février 2026, 1 900 appareils sur 3 400), dont une majorité de Galaxy A2x et A3x, pas des flagships.

## Ce qui se passe

One UI a deux mécanismes distincts, et les deux nous touchent :

1. **"Mettre en veille les applications inutilisées"** (activé par défaut) : une app non ouverte pendant 3 jours passe en veille profonde. Les push silencieux ne la réveillent plus, le service de suivi ne redémarre pas au boot. Un chauffeur qui prend un week-end de trois jours revient lundi avec une app qui ne synchronise plus tant qu'il ne l'ouvre pas à la main.

2. **Adaptive Battery** : même avec l'app ouverte tous les jours, le service au premier plan de suivi GPS (`TrackingForegroundService`) est tué au bout de 30 à 90 minutes d'écran éteint sur certains modèles (A25, A34 confirmés, S23 jamais). Pas de log, pas de callback, le service disparaît. On le voit côté serveur : positions qui s'arrêtent net puis reprennent quand l'écran se rallume.

## Ce qu'on a fait (HF-1400, app 4.6)

- Écran `BatteryExemptionScreen` affiché une fois à la première connexion et à nouveau si on détecte le problème : explique en une phrase, bouton qui ouvre `ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS` (autorisé sur le Play Store pour les apps de suivi de flotte, on l'a justifié dans la déclaration de permissions), et un second bouton vers les réglages Samsung "Applications jamais mises en veille" via un intent vers `com.samsung.android.lool` avec repli sur les réglages de l'app.

- Détection : si `TrackingForegroundService` a été démarré et que l'app n'a pas reçu de position depuis 20 minutes alors que l'écran est éteint et que la dernière position avait une vitesse supérieure à 10 km/h, on marque `suspected_kill` en local et on remonte un événement de télémétrie `tracking.suspected_kill` avec le modèle. C'est cette mesure qui a permis de savoir quels modèles étaient touchés.

- `WorkManager` avec une tâche périodique de 15 minutes qui vérifie que le service tourne et le redémarre sinon. Fonctionne contre Adaptive Battery, pas contre la veille profonde (WorkManager est lui aussi suspendu).

## Résultats

Avant 4.6 : `tracking.suspected_kill` sur 38 % des journées de conduite Samsung. Après, avec l'exemption acceptée : 6 %. Sans exemption acceptée (environ 30 % des chauffeurs ignorent l'écran) : 31 %. On relance l'écran tous les 14 jours tant qu'elle n'est pas accordée.

## Ce qu'on ne fait pas

Pas de notification permanente "L'app fonctionne" pour maintenir le service : les chauffeurs la trouvent intrusive et One UI la tue aussi. Pas de `AlarmManager.setExactAndAllowWhileIdle` en boucle, ça vide la batterie et ça n'est pas mieux.

Voir [[gps-tracking-battery-budget]] pour le budget global et [[device-quirk-huawei-no-gms]] pour l'autre famille d'appareils à problèmes.
