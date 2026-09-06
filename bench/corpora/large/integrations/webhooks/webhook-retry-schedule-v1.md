---
name: webhook-retry-schedule-v1
description: Ancien calendrier de relance (janvier à mai 2026) : 7 tentatives sur 18 h sans gigue ni pause par hôte, remplacé après mesure des livraisons mortes
type: reference
status: archived
superseded_by: [[webhook-retry-schedule-v2]]
verified: 2026-02-12
---

# Relances des webhooks, version 1 (jusqu'en mai 2026)

Le calendrier livré avec l'outbox transactionnelle en janvier 2026 et remplacé par [[webhook-retry-schedule-v2]] le 2026-05-12. Gardé pour lire les tickets et les tableaux de bord de la période.

## Le calendrier

Sept tentatives. Délais après chaque échec : 30 s, 2 min, 10 min, 30 min, 1 h, 4 h, 12 h. Cumul d'environ 18 h. Après la septième, `DEAD` et e-mail à l'admin de l'organisation. Cinquante `DEAD` sur sept jours désactivaient l'abonnement.

Pas de gigue : deux livraisons échouées à la même seconde revenaient à la même seconde.

Pas de pause par hôte : pendant une panne réseau chez un client, chaque livraison consommait ses tentatives indépendamment. Un client avec 500 livraisons en attente et un serveur arrêté 20 heures perdait les 500.

## Ce qu'on a observé

Entre janvier et avril 2026, environ 1 400 livraisons `DEAD` par semaine, dont la majorité chez des clients dont le serveur avait été arrêté un week-end. Les lundis, le support recevait des demandes de rejeu de plage sur 48 h, ce qui a fini par motiver l'outil de rejeu puis le nouveau calendrier.

Le pic de retour après une panne courte a été mesuré une fois : 2 100 livraisons vers un même hôte en 4 secondes, après une interruption de 10 minutes chez un chargeur ; leur serveur a répondu 503 à la moitié, qui sont reparties en échec pour 30 minutes. C'est l'origine de la gigue.

## Ce qui a été gardé

Le timeout de 10 s, la règle du 410, le comptage pour la désactivation automatique. La v2 a changé le nombre, les délais, la gigue et ajouté la pause par hôte, rien d'autre.
