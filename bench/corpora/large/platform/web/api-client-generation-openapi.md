---
name: api-client-generation-openapi
description: TypeScript API types are generated from the API's openapi/v2.json with openapi-typescript at build time, the fetch client is hand-written (src/api/http.ts) with auth refresh, error envelope parsing and the feature header
type: reference
status: active
verified: 2026-02-05
---

# Client API du front

## Types générés, client écrit à la main

`pnpm api:types` télécharge `public/openapi/v2.json` (et `internal.json` pour les routes `/internal/*`) depuis l'API de staging (variable `HF_API_SPEC_URL`) et lance `openapi-typescript` vers `src/api/generated/v2.d.ts`. Le fichier est commité, pour que le build ne dépende pas du réseau. Une CI nocturne régénère et ouvre une PR si le diff n'est pas vide, ce qui est notre alarme "le contrat a bougé".

Les types de chemins et de réponses viennent de là : `paths['/v2/loads']['get']['responses']['200']['content']['application/json']`. Des alias lisibles sont maintenus à la main dans `src/api/types.ts` (`export type Load = components['schemas']['Load']`), parce que personne ne veut écrire la forme longue.

Pas de client généré. Les générateurs testés produisaient soit des classes lourdes, soit un client `fetch` sans gestion de l'authentification. Le nôtre fait 180 lignes.

## `src/api/http.ts`

`request<TPath, TMethod>(path, { method, params, body })` :

1. Ajoute `Authorization` avec le jeton d'accès en mémoire, `X-Halden-Features` avec la liste de `src/api/features.ts`, `Accept-Language` avec la locale.

2. Sur 401 avec `type: .../errors/token-expired`, appelle `POST /v2/auth/refresh` une seule fois (mutex module, les appels concurrents attendent le même rafraîchissement), puis rejoue la requête. Un second 401 déconnecte.

3. Sur toute réponse non 2xx, parse l'enveloppe d'erreur et lève `ApiError` avec `type`, `status`, `detail`, `errors[]`, `traceId`. C'est `ApiError` que les formulaires et les toasts consomment, voir [[forms-react-hook-form-zod]] et [[error-boundary-and-toasts]].

4. Sur 429, lit `Retry-After` et lève `RateLimitedError` avec le délai. TanStack Query ne réessaie pas dessus.

5. Timeout 20 s via `AbortController`, 60 s pour les exports.

## Utilisation avec TanStack Query

Chaque ressource a un fichier `src/api/loads.ts` avec des fonctions `getLoads(filters)`, `getLoad(id)`, `assignLoad(id, carrierId)` qui appellent `request`, et des hooks `useLoads(filters)`, `useLoad(id)`, `useAssignLoad()` qui portent les clés de requête et les invalidations. Les composants importent les hooks, jamais `request`. Voir [[dispatch-board-state-zustand]].

## Gestion des versions

Quand l'API ajoute un champ optionnel, les types générés changent et rien ne casse. Quand elle ajoute un comportement derrière un nom de fonctionnalité (`X-Halden-Features`), on ajoute le nom dans `features.ts` dans la PR qui utilise le comportement, et on le retire quand l'API annonce qu'il est devenu le défaut.

Quand l'API supprime ou renomme, la CI nocturne ouvre la PR de types et `tsc` échoue dessus. C'est voulu : l'échec de compilation est la liste des endroits à corriger.

## Le `trace_id`

Toute `ApiError` porte `traceId`. Le toast d'erreur affiche "Code support : 0a1f3c9e" (les 8 premiers caractères) et le bouton copier met le `trace_id` complet dans le presse-papiers. Le support cherche ce code dans les logs.
