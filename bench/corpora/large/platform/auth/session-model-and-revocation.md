---
name: session-model-and-revocation
description: Une session est une ligne de auth.sessions liée au refresh token; révocation par le set Redis auth:revoked lu à chaque requête, 12 h web, 30 j app conducteur
type: reference
status: active
verified: 2026-05-06
---

## Le modèle

Une connexion réussie crée une ligne dans `auth.sessions` et une famille de refresh tokens rattachée à `session_id`. Le JWT d'accès (15 minutes) porte `sid` (l'identifiant de session) en plus de `sub` et des rôles. Une session a une durée maximale (`expires_at`, 12 h pour le web, 30 jours pour l'app conducteur, 8 h pour les comptes support) et une durée d'inactivité (`last_seen_at` mis à jour au plus une fois par minute, expiration après 2 h d'inactivité sur le web, jamais sur l'app conducteur, qui est un appareil de travail).

L'ancien modèle de cookie de session PHP et ses limites sont dans [[legacy-php-session-cookies]].

## Révocation

Un JWT d'accès est valable 15 minutes sans qu'on puisse le rappeler ; c'est le compromis du format. Pour que « déconnexion » ou « changement de mot de passe » agissent tout de suite, l'API vérifie à chaque requête que le `sid` du token n'est pas dans le set Redis `auth:revoked` (un `SISMEMBER`, 0,2 ms, sur le Redis de l'API). Une révocation écrit `revoked_at` et `revoke_reason` sur la ligne, et ajoute le `sid` au set avec une expiration égale au temps restant du plus long token d'accès possible (15 minutes) plus une marge : 20 minutes. Après ça, le refresh échoue de toute façon parce que la ligne est révoquée.

Motifs de révocation, dans `revoke_reason` : `logout`, `logout_all`, `password_changed`, `mfa_reset`, `admin`, `support`, `reuse_detected` (posé par la détection de réutilisation du refresh token), `device_revoked`, `stuffing_response` (utilisé une fois, décembre 2025, sur 2 300 sessions).

## Déconnexion partout

`POST /v1/me/sessions/revoke-all` révoque toutes les sessions de l'utilisateur sauf la courante (option `include_current`). Un changement de mot de passe le fait d'office, un `mfa_reset` aussi. Le set Redis prend 2 300 membres en une transaction sans que personne le voie ; pour la réponse à un incident, la commande `bin/console auth:sessions revoke --user <id> --reason admin` ou `--where "created_at > ..."` fait la même chose en masse.

## Liste des sessions

`GET /v1/me/sessions` renvoie les sessions actives avec `ip` (tronquée au /24 pour l'affichage), une étiquette d'appareil dérivée du `user_agent` au moment de la connexion (on ne stocke que le hash après), la date de première connexion et de dernière activité, et le drapeau `current`. La page « appareils connectés » de l'outil dispatch et l'écran équivalent de l'app conducteur lisent ça. En mai 2026, 6 % des utilisateurs web ont ouvert la page au moins une fois, et 1 400 sessions ont été révoquées par leurs propriétaires, dont une centaine avec un ticket « ce n'est pas moi » qui ont toutes été étudiées ([[auth-events-audit-log]] pour la trace).

## Nettoyage

Les lignes expirées ou révoquées depuis plus de 30 jours sont supprimées chaque nuit par `auth:sessions purge`, par lots de 10 000. La table tient à 1,3 M de lignes actives et 0,4 M en attente de purge. L'index utile est `(user_id, revoked_at)` ; l'index `(expires_at)` sert à la purge.

## Ce qui est volontairement simple

- Pas de « session glissante » sur l'app conducteur : 30 jours, puis un nouvel OTP. On a mesuré que 92 % des chauffeurs actifs se connectent au moins une fois par semaine de toute façon, et un OTP mensuel est le prix de ne pas avoir de session éternelle sur un téléphone perdu.

- Pas de liaison de session à l'IP. Les chauffeurs changent d'IP dix fois par jour. La liaison est à l'appareil ([[device-trust-and-remember-me]]) pour le web, et au `device_id` de l'app pour les chauffeurs.

- Un seul Redis pour la révocation, celui de l'API, sans réplique dédiée. S'il tombe, l'API refuse les requêtes plutôt que d'accepter un token peut-être révoqué (`AUTH_REVOCATION_FAIL_CLOSED=true`). Ça a coûté 4 minutes d'indisponibilité en février 2026 lors d'un basculement Redis, et on a choisi de garder ce comportement.
