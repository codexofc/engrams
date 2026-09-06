---
name: webhook-auto-disable-and-dead-letters
description: Livraisons DEAD : e-mail à tous les admins (HF-3121), désactivation à 50 DEAD sur 7 jours ou sur 410, rapport hebdomadaire, pas de file séparée
type: project
status: active
verified: 2026-06-30
---

# Livraisons mortes et désactivation automatique

Une livraison qui a épuisé ses tentatives ([[webhook-retry-schedule-v2]]) passe en `DEAD`. Elle reste dans `sys_outbox` avec son `last_error` pendant 30 jours, visible dans l'écran des livraisons et par `hfctl`, rejouable ([[webhook-replay-tool]]). Il n'y a pas de table de lettres mortes à part : le statut suffit, et l'index partiel exclut les `DEAD` du chemin chaud.

## Notification

Au premier `DEAD` d'une journée pour un abonnement, un e-mail part aux admins de l'organisation : événement, `delivery_id`, `last_error` en clair, lien vers l'écran. Un seul e-mail par abonnement et par jour, sinon un endpoint mort génère un e-mail par événement.

Jusqu'en mars 2026 l'e-mail partait à l'utilisateur qui avait **créé** l'abonnement. Dans un cas sur trois cette personne avait quitté l'entreprise ou changé de poste, et l'organisation découvrait la coupure des semaines plus tard par ses propres utilisateurs. HF-3121 envoie à tous les `org_admin`, plus à l'adresse technique de l'abonnement si elle est renseignée (`PATCH` avec `notify_email`). Depuis, le délai médian entre première livraison morte et action du client est passé de 9 jours à 1 jour.

## Désactivation automatique

Deux déclencheurs :

- **50 `DEAD` sur 7 jours glissants** pour un même abonnement : `status = DISABLED`, `disabled_reason = too_many_failures`. Les événements suivants ne sont plus mis en outbox pour cet abonnement, donc pas de rattrapage possible au-delà de ce qui était déjà en file. C'est volontaire : un endpoint mort depuis une semaine n'a pas besoin de 40 000 lignes en attente.

- **Un 410 Gone** : désactivation immédiate, `disabled_reason = gone`. C'est le moyen propre pour un intégrateur de dire « arrêtez » sans se connecter chez nous. Utilisé par deux intégrateurs qui décommissionnent des environnements ainsi.

La réactivation se fait depuis l'écran (bouton « Réactiver », qui exige que l'URL réponde au `ping`) ou par `hfctl webhooks enable`. Les compteurs repartent à zéro.

## Ce qui est compté

Seules les tentatives réelles comptent. Un abonnement en pause n'accumule rien. Les livraisons `FAILED` non encore mortes ne comptent pas. Les hôtes en pause ne consomment pas de tentatives, donc une panne réseau de 20 h chez le client ne produit plus de `DEAD` à elle seule, ce qui était le cas en v1 et déclenchait des désactivations pour un week-end de maintenance.

## Rapport hebdomadaire

Le lundi à 8 h, une requête envoie au canal support la liste des abonnements avec plus de 20 `DEAD` sur la semaine, avec l'organisation, le plan et la première `last_error`. Le support prévient les Business et Enterprise avant qu'ils ne demandent ; les Starter reçoivent l'e-mail automatique et c'est tout. Entre 8 et 25 lignes par semaine en 2026.

## Chiffres

Avant HF-3121 et la v2 des relances (janvier à avril 2026) : 30 à 40 désactivations automatiques par mois, dont un tiers réactivées dans la semaine, ce qui veut dire qu'elles n'auraient pas dû arriver. Depuis mai : 8 à 12 par mois, presque toutes des URL réellement abandonnées.

## Ce qu'on ne fait pas

Pas de file de lettres mortes séparée, pas d'export automatique des `DEAD` vers un stockage objet (demandé une fois, l'intégrateur voulait « tout garder » ; le rejeu sur 30 jours couvre le besoin réel). Pas de désactivation manuelle par le support sans demande écrite du client, sauf abus (le script de rejeu toutes les 5 minutes).
