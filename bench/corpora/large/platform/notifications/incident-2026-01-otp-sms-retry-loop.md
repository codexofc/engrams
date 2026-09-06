---
name: incident-2026-01-otp-sms-retry-loop
description: Janvier 2026, une boucle de retry côté app conducteur a demandé 14 300 OTP SMS à 300 chauffeurs en 70 minutes, 2 100 EUR de SMS, la limite par destinataire n'était pas branchée, HF-4131
type: project
status: active
verified: 2026-02-10
---

# Incident 2026-01-20 : boucle d'OTP SMS

## Impact

De 06:10 à 07:20 UTC, 14 300 SMS d'OTP envoyés à 312 chauffeurs, soit 46 par chauffeur en moyenne, le maximum observé étant 118. Coût : 2 100 EUR facturés par Bipline. Aucun accès frauduleux, aucun code valide n'a été utilisé autrement que par son destinataire. Une trentaine de chauffeurs ont appelé leur transporteur, huit tickets support, un transporteur a menacé de désinstaller l'app pour toute sa flotte.

## Chronologie (UTC)

- 06:10 : la version 5.2.0 de l'app conducteur est poussée à 20 % des utilisateurs. Elle contient une refonte de l'écran de connexion. Quand la réponse de `POST /v1/auth/otp/request` met plus de 3 secondes, le nouvel écran renvoie la requête, sans limite, sans attente croissante.

- 06:12 : l'API de l'auth répond en 3,5 s en médiane au pic du matin (le service KYC Verifid est interrogé de façon synchrone à la première connexion du jour, ce qui était déjà un problème connu). Les téléphones en 5.2.0 renvoient donc la requête en boucle.

- 06:25 : Bipline répond 429 sur le sémaphore à 50 requêtes par seconde. `notify-worker` met les livraisons en retry (30 s, 5 min, 30 min). Le graphe de file `sms` monte à 9 000.

- 06:40 : `NotifyBacklog` (`warn` à l'époque) est vu par l'astreinte plateforme, qui pense d'abord à une panne fournisseur.

- 06:55 : `notifications:trace` sur trois livraisons montre le même `recipient_id` des dizaines de fois, tous en `auth.otp`, tous avec un `user_agent` en 5.2.0. La cause est identifiée.

- 07:05 : le déploiement de 5.2.0 est stoppé et ramené à 0 % (déploiement progressif côté magasins). Le producteur est coupé au niveau de l'API : `auth.otp` limité à 3 par 10 minutes et par utilisateur, en dur dans le contrôleur, en attendant la vraie limite.

- 07:20 : la file `sms` est purgée des `auth.otp` de plus de 5 minutes (`notifications:purge-queue --event auth.otp --older-than 5m`, commande écrite pour l'occasion et gardée) : 6 800 messages abandonnés. Un OTP expiré envoyé quand même est le pire résultat possible, on paie et on inquiète.

## Causes

1. L'app conducteur réessayait sans limite. Un `retry` sans `backoff` ni plafond sur une requête qui a un effet (envoyer un SMS) est la cause première. L'écran a été corrigé en 5.2.1 (un seul renvoi automatique, puis un bouton avec compte à rebours de 60 s).

2. La limite par destinataire, prévue par la refonte, n'était pas branchée dans le worker : la classe existait, l'appel n'y était pas. Elle l'est depuis, voir [[per-recipient-rate-limits]].

3. Le seau OTP côté API n'existait pas : `POST /v1/auth/otp/request` acceptait toute requête authentifiée par un numéro valide. L'auth a ajouté sa propre limite, distincte de celle des notifications.

4. Le 429 de Bipline était traité comme une erreur transitoire et mis en retry avec les mêmes délais qu'une erreur réseau. Un 429 sur un OTP est maintenant une erreur finale : le code a une durée de vie de 5 minutes, le renvoyer 30 minutes plus tard n'a aucun sens. `BiplineSmsClient` lève `RateLimitedByProvider`, non réessayable pour les types d'événement avec `ttl` dans `event_types.yaml`.

## Ce qui a changé

- `event_types.yaml` a un champ `ttl` : `auth.otp: 5m`, `load.assigned: 6h`, les digests `24h`. Le worker abandonne une livraison dont le `ttl` est dépassé au moment de l'envoi, avec `status = suppressed`, `suppression_reason = 'expired'`. Ce seul champ aurait divisé l'impact par deux.

- `NotifyBacklog` est passé de `warn` à `page` au-dessus de 20 000, et un nouveau signal `NotifyRecipientBurst` (`page`) compte les destinataires distincts ayant reçu plus de 10 livraisons en 10 minutes : au-dessus de 20 destinataires, quelqu'un est appelé. Ce signal aurait sonné à 06:20.

- Le tableau de bord a un panneau « livraisons par type d'événement et par version de client », parce que la version était la clé de compréhension et qu'il a fallu la chercher à la main.

- Bipline nous a accordé un geste de 800 EUR après explication ; les 1 300 restants sont dans la ligne SMS de janvier ([[sms-country-routing-and-unit-costs]] pour la référence mensuelle).

## Ce qu'on retient

Une notification est une action qui coûte et qu'on ne peut pas reprendre. Toute limite de débit qui n'a pas été vue en train de refuser quelque chose en staging doit être considérée comme absente.
