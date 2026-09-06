---
name: api-error-envelope-convention
description: Every non-2xx response of the Halden API returns the same JSON envelope (type, title, status, detail, errors[], trace_id), decided in HF-1187
type: reference
status: active
verified: 2026-03-14
---

# Enveloppe d'erreur de l'API

Depuis HF-1187 (novembre 2025), toute réponse non 2xx de `halden-api` a la même forme, inspirée de RFC 7807 mais sans le `Content-Type: application/problem+json` (le client Flutter de l'époque ne le gérait pas, voir le ticket HF-1190 côté mobile).

## Forme

```json
{
  "type": "https://docs.halden.example/errors/load-not-biddable",
  "title": "Load is not open for bids",
  "status": 409,
  "detail": "Load L-2026-004512 is in state DISPATCHED",
  "errors": [],
  "trace_id": "0a1f3c9e8b7d4e2f"
}
```

## Règles

- `type` est une URL stable, jamais un message. La liste est dans `config/api/error_types.yaml`, un slug par constante de `App\Api\ErrorType`. Ajouter un slug = ajouter la constante, sinon `ErrorTypeConsistencyTest` échoue.

- `errors[]` ne sert qu'aux erreurs de validation (422). Chaque entrée est `{ "path": "pickup.window.start", "code": "date_in_past", "message": "..." }`. Le `path` suit la structure du JSON envoyé, pas le nom du champ Symfony Form (on ne passe plus par les Forms depuis Symfony 6.4, tout est DTO + `#[MapRequestPayload]`).

- `trace_id` vient de l'en-tête `traceparent` si présent, sinon il est généré dans `TraceIdSubscriber`. C'est cet identifiant qu'on demande au support quand un client se plaint. Il est aussi dans les logs Loki sous le label `trace_id`.

- `detail` peut contenir des identifiants métier (numéro de chargement, immatriculation) mais jamais d'e-mail ni de nom de personne. `ErrorEnvelopeListener` passe le `detail` dans `PiiScrubber` avant sérialisation, ce qui coûte environ 40 µs par erreur, mesuré sur un 409 simple.

## Cas particuliers

- Les 401 gardent l'en-tête `WWW-Authenticate: Bearer realm="halden"` pour les vieux clients, mais le corps est l'enveloppe.

- Les 429 ajoutent `Retry-After` en secondes. Le mobile lit cet en-tête et pas le corps, voir [[rate-limiting-per-carrier]].

- Les 500 ont un `detail` fixe ("Internal error") en prod et le message d'exception en `dev` et `staging`. Le flag est `APP_ERROR_DETAIL_VERBOSE`, pas `APP_DEBUG` (on a eu un staging avec `APP_DEBUG=1` en 2025 et le profiler qui mangeait 30 % du CPU).

## Ce qu'on ne fait pas

On avait envisagé de mettre le code d'erreur dans un en-tête `X-Halden-Error`. Refusé : les proxys des transporteurs (au moins deux qui passent par un filtrage sortant d'entreprise) suppriment les en-têtes inconnus. Tout dans le corps.

Le mapping exception vers enveloppe est dans `App\Api\Exception\ExceptionEnvelopeMapper`. Une exception métier qui ne l'étend pas `DomainException` sort en 500, c'est voulu : une exception inconnue est un bug, pas une erreur client.
