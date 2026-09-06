---
name: sync-polling-interval-old
description: Historical polling sync of the driver app (full GET every 60 s), replaced by push-triggered delta sync in app 4.5
type: reference
status: archived
superseded_by: [[sync-push-triggered-delta]]
verified: 2025-10-20
---

# Synchronisation par polling (jusqu'à l'app 4.4)

Jusqu'en octobre 2025, l'app faisait un `GET /internal/mobile/loads` complet toutes les 60 secondes quand elle était au premier plan, et toutes les 15 minutes en arrière-plan via `WorkManager` (Android) et `BGAppRefreshTask` (iOS).

Ce que ça coûtait :

- 3 400 appareils × 1 requête/minute au premier plan = environ 1 200 requêtes par minute en pointe pour des réponses identiques à 98 %.

- Réponse moyenne de 40 Ko (tous les chargements des 30 derniers jours, sans delta), soit environ 2 Go par jour de trafic sortant pour rien.

- Latence de prise en compte d'une affectation : jusqu'à 60 s au premier plan, jusqu'à 15 minutes en arrière-plan, et les dispatchers appelaient les chauffeurs pour leur dire de rafraîchir.

Le remplacement est décrit dans [[sync-push-triggered-delta]]. L'endpoint `GET /internal/mobile/loads` sans curseur existe encore pour la synchronisation initiale.
