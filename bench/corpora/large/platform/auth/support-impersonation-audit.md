---
name: support-impersonation-audit
description: Le support peut ouvrir une session « en tant que » un utilisateur pour 30 minutes, lecture seule par défaut, écriture avec un motif et un ticket, bandeau visible, tout journalisé dans auth_events avec l'agent, relu chaque mois par échantillon
type: reference
status: active
verified: 2026-03-30
---

## Pourquoi ça existe

« Je ne vois pas mon chargement » se règle en regardant l'écran de l'utilisateur. Avant octobre 2025, le support le faisait avec un compte administrateur global qui voyait tout et pouvait tout faire, sans trace de ce qu'il regardait. L'extraction de `auth-svc` ([[auth-service-overview]]) a été l'occasion de remplacer ça par une impersonation explicite, bornée et journalisée (HF-4308).

## Le mécanisme

`POST /v1/auth/impersonate {"user_id": ..., "ticket": "SUP-...", "mode": "read" | "write", "reason": "..."}`, réservé au rôle `support`, avec un code TOTP frais exigé (`mfa_at` dans les 5 minutes, appareil de confiance ou non). Il crée une session pour l'utilisateur cible avec trois particularités :

- `sessions.impersonated_by = <agent user_id>` et `expires_at = now + 30 min`, non prolongeable. Pour continuer, l'agent recommence, avec un nouveau code TOTP ;

- le JWT porte une revendication `act` (l'agent) et `imp_mode`. L'API refuse toute requête non `GET` quand `imp_mode = read`, et en mode `write` elle refuse quand même quatre familles d'opérations : changement de coordonnées bancaires, changement de mot de passe ou de MFA, suppression de compte, acceptation d'une enchère. Ces quatre-là, l'utilisateur les fait lui-même, ou elles ne se font pas ;

- l'outil dispatch et l'app web expéditeur affichent un bandeau rouge « session support (prénom de l'agent), ticket SUP-…, expire à hh:mm », visible par l'agent. L'utilisateur, lui, reçoit un e-mail `auth.security_alert` : « un membre du support a accédé à votre compte pour le ticket SUP-… », avec le nom de l'agent et la durée. Ce mail a été débattu, il est gardé : la personne dont on regarde le compte a le droit de le savoir.

Le mode `write` demande un `reason` d'au moins 20 caractères et un ticket dont l'existence est vérifiée par l'API du support. 8 % des impersonations sont en `write`, presque toutes pour corriger une adresse de site de livraison mal saisie par un expéditeur qui n'y arrive pas au téléphone.

## Ce qui est journalisé

Dans `auth_events` ([[auth-events-audit-log]]) : `impersonation.started` (agent, cible, ticket, mode, motif), `impersonation.ended` (durée, nombre de requêtes API faites dans la session, par méthode HTTP), et chaque requête d'écriture de la session dans le journal d'audit de l'API avec `actor = agent, on_behalf_of = cible`. Les requêtes de lecture ne sont pas journalisées une à une (le volume ne le justifie pas), seulement comptées.

## Revue mensuelle

Le premier lundi du mois, la responsable du support et une personne de la plateforme prennent 20 impersonations au hasard du mois écoulé et vérifient : le ticket existe et parle bien de cet utilisateur, le mode est cohérent avec le ticket (un ticket « je ne vois pas » en `write` est une question), la durée est raisonnable (une session de 30 minutes fermée à 29 pour un ticket de « mot de passe oublié » aussi). Depuis octobre 2025, six revues, trois remarques : deux `write` qui auraient dû être `read`, une impersonation d'un compte de test par un agent qui essayait la fonctionnalité (autorisé, mais avec un ticket, désormais).

## Chiffres (mars 2026)

- 1 900 impersonations, 12 agents, médiane 6 minutes, 92 % en `read`.

- 4 utilisateurs ont répondu au mail de notification pour demander pourquoi ; les quatre avaient un ticket ouvert et l'avaient oublié.

- 0 refus par l'API d'une des quatre familles interdites, ce qui veut dire que personne n'a essayé, ce qui est le résultat qu'on veut.

## Ce qu'on a écarté

- L'impersonation « silencieuse » sans mail à l'utilisateur, demandée pour les cas de fraude où on ne veut pas prévenir : ces cas passent par le back-office de conformité, qui lit les données sans ouvrir de session au nom de l'utilisateur, et c'est une autre trace.

- Un rôle « support senior » qui pourrait faire les quatre opérations interdites : non, la liste est courte pour que la réponse soit non.
