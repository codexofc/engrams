---
name: webhook-payload-versioning-v2-v3
description: Versions de payload webhook : v2 plate par défaut, v3 avec objets imbriqués, sequence et meta, règle de nullabilité, v2 servie jusqu'à fin 2027
type: reference
status: active
verified: 2026-07-20
---

# Versions de payload : v2 et v3

Chaque abonnement a un `payload_version`. Chaque ligne d'outbox porte le `schema_version` avec lequel elle a été construite. Une version, c'est une forme de JSON, pas une liste d'événements : les mêmes événements existent dans les deux.

## v2 (depuis janvier 2026, par défaut)

Forme plate. Exemple `load.dispatched` :

```json
{
  "event": "load.dispatched",
  "occurred_at": "2026-05-04T08:12:31Z",
  "load_id": "0190a1c2-...",
  "reference": "PO-88214",
  "status": "DISPATCHED",
  "carrier_id": "0189f0...",
  "carrier_name": "Kowalczyk Logistik",
  "driver_id": null,
  "driver_name": null,
  "pickup_window_start": "2026-05-05T06:00:00Z",
  "schema_version": 2
}
```

Tout en `snake_case`, dates en UTC ISO 8601 avec `Z`, identifiants en UUID. Les champs sont stables : on **ajoute** des champs en v2 (un intégrateur doit ignorer ce qu'il ne connaît pas, c'est dans le guide), on n'en retire ni n'en renomme jamais.

## v3 (production depuis avril 2026, HF-3102)

Ce qui change :

- Les relations deviennent des objets : `carrier: { id, name, country }`, `driver: { id, name, phone_last4 }` ou `null`, `shipper: { id, name }`.

- Un champ `sequence` par chargement, entier croissant, pour ordonner les événements d'un même `load_id` côté client (voir [[webhook-ordering-and-sequence-numbers]]).

- Un objet `meta` : `{ delivery_id, replay_of, reason, ticket }`. `reason` porte par exemple `delivery_voided` sur un `load.in_transit` qui suit une correction support.

- `occurred_at` gagne les millisecondes.

- Les montants deviennent `{ amount_cents, currency }` au lieu de `amount` et `currency` séparés.

Pas de nouveaux événements en v3. Les `event` sont identiques.

## Règle de nullabilité

Écrite après [[webhook-incident-2026-06-payload-v3-null-driver]] : un champ nullable est présent avec la valeur `null`. Jamais absent, jamais `{}`, jamais `[]` à la place de `null`. Une liste vide est `[]` quand le champ est une liste (les `documents` d'un chargement sans document), `null` quand c'est une relation absente. Les schémas JSON publiés par version le disent champ par champ.

## Changer de version

`PATCH /v2/webhooks/subscriptions/{id}` avec `payload_version: 3`. Les lignes déjà en outbox gardent leur `schema_version` ; les nouvelles sont construites en v3. Pendant quelques minutes, l'intégrateur peut recevoir les deux : il lit `schema_version` dans le corps et branche. Le guide recommande de déployer le code qui accepte les deux, puis de basculer, jamais l'inverse.

Un rejeu ([[webhook-replay-tool]]) renvoie la version d'origine de la livraison, pas la version courante de l'abonnement.

## Dépréciation

v2 reste servie **au moins jusqu'au 2027-12-31**. Six mois avant tout arrêt, e-mail aux admins des organisations concernées et bandeau dans l'écran. Il n'y a pas de date d'arrêt aujourd'hui ; 70 % des abonnements sont en v2 en juillet 2026 et ça ne presse pas. Une v4 n'est pas prévue ; les besoins remontés (filtrage par champ, batching) se règlent par des options d'abonnement, pas par une nouvelle forme.

## Ajouts en cours de version

Un champ ajouté est annoncé dans le changelog intégrateur 30 jours avant, avec un exemple. Depuis juin 2026, tout changement de forme, même un ajout, passe par une fixture écrite à la main et revue (`tests/Webhook/fixtures/v2/`, `.../v3/`). La liste des champs par événement et par version est générée depuis ces fixtures vers le site de documentation à chaque déploiement.

## Ce qu'on ne fait pas

Pas de payload « complet » (tout l'objet chargement) : les événements portent ce qui a changé et les identifiants, le reste se lit par l'API. Pas de payload configurable par abonnement (choisir ses champs) : deux formes à maintenir, c'est déjà une de trop.
