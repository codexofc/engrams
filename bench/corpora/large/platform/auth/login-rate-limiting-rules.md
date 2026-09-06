---
name: login-rate-limiting-rules
description: LoginThrottle applique quatre compteurs avant tout hachage, 20 tentatives par IP et par heure, 10 par e-mail et par heure, 5 par couple, plus un mode dégradé global au-delà de 40 % d'échecs, tout dans Redis, réponses indistinguables
type: reference
status: active
verified: 2026-04-09
---

## Les compteurs

`LoginThrottle::check(LoginAttempt $a): Verdict` s'exécute au tout début de `POST /v1/auth/login`, avant la recherche de l'utilisateur et avant Argon2id (140 ms, voir [[password-policy-and-argon2]]). Quatre compteurs en fenêtre glissante dans Redis (`auth:thr:<type>:<clé>`), script Lua, comme le limiteur des notifications dont le code a été repris.

### Les quatre compteurs

| Compteur | Clé | Limite | Fenêtre | Ce qu'il attrape |
|---|---|---|---|---|
| par IP | `ip:<adresse>` | 20 échecs | 1 h | une IP qui essaie beaucoup de comptes |
| par e-mail | `email:<sha256 normalisé>` | 10 échecs | 1 h | beaucoup d'IP sur un compte |
| par couple | `pair:<ip>:<sha256>` | 5 échecs | 15 min | la force brute classique |
| global | `global` | taux d'échec | 10 min | la vague, quand les autres sont contournés |

Seuls les échecs comptent. Une réussite remet à zéro le compteur par couple et par e-mail, pas celui par IP (une IP qui réussit un compte sur cinquante est toujours suspecte). Un utilisateur légitime derrière le NAT d'un bureau de dispatch partage son compteur IP avec ses collègues : 20 échecs par heure pour un bureau de 30 personnes n'a jamais été atteint en conditions normales (mesuré : maximum 7 sur un mois pour le plus gros bureau).

Le compteur global ne bloque pas : au-delà de 40 % d'échecs sur 10 minutes avec au moins 1 000 tentatives, il passe le service en mode dégradé, où les limites par IP et par e-mail sont divisées par deux et où toute réussite depuis une IP ayant au moins un échec sur un autre compte dans l'heure déclenche le défi MFA même sur un appareil de confiance. Le mode dégradé s'est enclenché deux fois depuis décembre 2025 (janvier et avril 2026, des vagues plus petites que celle de [[incident-2025-12-credential-stuffing]], 200 000 et 80 000 tentatives), sans effet visible pour les utilisateurs normaux.

## La réponse

Une tentative refusée par le limiteur reçoit `401 {"error": "invalid_credentials"}`, la même réponse qu'un mauvais mot de passe, avec le même délai (un hachage factice). Pas de `429`, pas de `Retry-After` : dire à l'attaquant qu'il est limité lui dit d'aller plus vite ailleurs, et dire à l'utilisateur légitime « trop de tentatives » sans lui dire combien de temps ne l'aide pas non plus. L'utilisateur légitime bloqué reçoit, lui, un e-mail `auth.security_alert` à la 10e tentative sur son compte : « des tentatives de connexion échouent sur votre compte, si c'est vous, réinitialisez votre mot de passe ». Le lien de réinitialisation n'est pas soumis au compteur par e-mail (il a le sien, 3 par heure, [[password-reset-flow]]).

## Ce que le limiteur ne fait pas

- Il ne verrouille pas le compte. Le verrouillage est une autre règle, sur d'autres signaux, décrite dans [[account-lockout-policy]]. Un compteur qui expire au bout d'une heure n'est pas un verrouillage, et c'est voulu : un attaquant qui peut verrouiller n'importe quel compte en dix tentatives a un déni de service gratuit.

- Il ne regarde pas la géographie ni la réputation d'IP. Les deux ont été essayés en janvier sur les journaux de décembre : la réputation aurait bloqué 40 % des IP de la vague et 2 % de nos utilisateurs légitimes (ceux derrière des VPN d'entreprise sortant par des ranges d'hébergeurs). Pas acceptable pour 40 %.

- Il ne s'applique pas à l'OTP des chauffeurs, qui a ses propres limites (3 demandes par 10 minutes par numéro, 5 vérifications par code) dans [[driver-otp-login-server-side]].

## Observabilité

`auth.throttle_rejected{counter}` par minute, panneau « Limiteur » du tableau `Auth / Login`. Le compteur qui rejette le plus en temps normal est `pair` (60 par jour, des gens qui se trompent), puis `email` (15 par jour, souvent des comptes partagés où deux personnes changent le mot de passe l'une après l'autre, voir [[abuse-shared-dispatcher-accounts]]). `ip` à plus de 100 par heure est le premier signe d'une vague, avant même le taux d'échec.

`bin/console auth:throttle show --email <adresse>` et `--ip <adresse>` affichent les compteurs courants ; `auth:throttle reset --email ...` les remet à zéro, avec ligne d'audit, pour le support quand un utilisateur légitime est bloqué et pressé. Une trentaine de fois par mois.
