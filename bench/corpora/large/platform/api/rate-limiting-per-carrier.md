---
name: rate-limiting-per-carrier
description: Per-organisation rate limits on the API (sliding window in Redis, 600 req/min for integrations, 1200 for driver apps per org), 429 with Retry-After, and the story of the Belgian carrier polling every 5 seconds
type: project
status: active
verified: 2026-03-30
---

# Rate limiting par organisation

Mis en place en HF-1230 (janvier 2026) après qu'un intégrateur d'un transporteur belge a déployé un connecteur TMS qui faisait `GET /v2/loads?status=OPEN` toutes les 5 secondes depuis 40 dépôts, soit 480 requêtes par minute pour une seule organisation, en permanence, week-end compris. Ce n'était pas dangereux mais ça représentait 11 % du trafic total pour 0,2 % des chargements.

## Règles

Limiteur Symfony `RateLimiter` avec le stockage Redis (`rate_limiter.storage: cache.rate_limiter`, pool dédié `redis://redis-ratelimit:6379`), politique `sliding_window`.

| Clé | Limite | Fenêtre |
|---|---|---|
| `org:<uuid>:integration` | 600 requêtes | 1 min |
| `org:<uuid>:driver` (somme de tous les chauffeurs de l'organisation) | 1200 requêtes | 1 min |
| `org:<uuid>:web` | 3000 requêtes | 1 min |
| `user:<uuid>:exports` | 20 requêtes | 1 h |
| `ip:<addr>:anonymous` (routes sans auth : login, jwks, health) | 60 requêtes | 1 min |

Le rôle vient du JWT (`role` claim), voir [[jwt-auth-and-refresh-tokens]]. Le limiteur s'exécute dans `RateLimitSubscriber` avec priorité 8 sur `kernel.request`, donc après l'authentification (priorité 16) et avant le contrôleur.

## Réponse

429, enveloppe standard avec `type: .../errors/rate-limited`, en-tête `Retry-After` en secondes entiers (arrondi au supérieur) et `X-RateLimit-Remaining` sur toutes les réponses, pas seulement les 429, pour que les intégrateurs puissent se réguler. Voir [[api-error-envelope-convention]].

Le mobile ne lit que `Retry-After`. Il suspend la synchronisation pendant ce délai et affiche rien à l'utilisateur, ce qui est le comportement voulu : un chauffeur ne peut rien faire d'une erreur de quota.

## Ce qui n'est pas limité

- Les webhooks sortants (c'est nous qui appelons).
- `POST /v2/tracking/positions` du mobile : on a essayé de le mettre dans le quota `driver` et les remontées GPS de 60 chauffeurs saturaient le quota de leur organisation en une minute. Il a son propre limiteur, 1 position par 10 s par chauffeur, appliqué côté serveur en ignorant silencieusement les positions trop proches (200 avec `accepted: false`), pas en 429.
- Le rôle `support`.

## Exemptions

`sys_feature_flags` a une clé `ratelimit.exempt_orgs` (liste d'UUID). Utilisée deux fois : une migration de données chez un chargeur, et le transporteur belge pendant les trois semaines qu'il a fallu à son intégrateur pour passer à un polling toutes les 2 minutes plus webhooks. Le flag est vérifié à chaque requête via le cache `app.flags` (TTL 30 s).

## Mesures

Depuis la mise en place, le trafic total a baissé de 9 % sans qu'aucun client fonctionnel ne soit touché. Le nombre de 429 en régime normal : environ 40 par jour, quasi tous sur `user:<uuid>:exports` par des dispatchers qui cliquent plusieurs fois sur "Exporter".
