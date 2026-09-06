---
name: analytics-events-naming
description: Product events named object.action in past tense with a fixed envelope, declared in the registry, versioned by adding properties
type: reference
status: active
verified: 2026-04-11
---

Convention en vigueur depuis février 2026 (HF-2440), qui remplace celle décrite dans [[analytics-events-v1-naming]].

## Nom

`<objet>.<action au passé>` : `load.posted`, `bid.placed`, `bid.withdrawn`, `bid.flagged_junk`, `invoice.issued`, `document.uploaded`, `verification.passed`. L'objet est au singulier, l'action au participe passé anglais. Pas d'écran ni de bouton dans le nom (`onboarding_page_button_clicked` était le pire exemple de l'ancienne convention) : on trace ce qui est arrivé au domaine, pas où l'utilisateur a cliqué. Les vues d'écran sont un seul événement `screen.viewed` avec la propriété `screen`.

## Enveloppe

Toujours présente, remplie par le SDK, jamais à la main :

- `event`, `event_version` (entier, 1 au départ),
- `occurred_at` en UTC, `received_at`,
- `actor` : `type` (`carrier`, `shipper`, `staff`, `system`) et `id`,
- `unit_ids` : `carrier_id`, `shipper_id`, `load_id`, `bid_id`, `invoice_id` quand ils ont un sens,
- `experiments` : liste des `flag:variant` actifs pour cet acteur au moment de l'événement (voir [[feature-flags-convention]]),
- `app` : `web_shipper`, `web_carrier`, `mobile_driver`, `backoffice`, `service`, avec sa version,
- `entity_code`.

## Propriétés

Spécifiques à l'événement, déclarées dans le registre `events/registry.yaml` avec leur type et une description. Un événement non déclaré est rejeté par le collecteur en staging et accepté avec un avertissement en production (compteur `analytics.undeclared_event`, qui doit rester à zéro). Les montants sont en cents entiers avec une propriété `currency`, les durées en millisecondes, les dates en UTC ISO 8601.

## Versionnage

On ajoute des propriétés, on n'en renomme ni n'en supprime. Un changement de sens d'une propriété est un nouvel événement ou un incrément de `event_version` avec les deux versions déclarées. L'entrepôt garde les deux et les analystes choisissent.

## Ce qui n'est pas un événement produit

- Les logs techniques (latence, erreurs HTTP) restent dans l'observabilité.
- Les données personnelles au-delà des identifiants : pas d'email, pas de nom, pas de numéro de téléphone dans les propriétés. Le collecteur rejette une propriété qui ressemble à un email.
- Les événements de domaine du bus interne (`LoadAwarded`, `InvoiceIssuedEvent`) sont une autre chose : ils font tourner le système. Un événement produit est émis en plus, par le même service, avec la convention ci-dessus, et l'entrepôt les rapproche par `load_id` ou `invoice_id`.

## Registre

`events/registry.yaml` est dans le dépôt commun, une entrée par événement avec `owner`, `since`, `properties`. La revue d'un nouvel événement demande une requête d'exemple qui l'utilise : un événement que personne ne sait comment interroger n'est pas ajouté.
