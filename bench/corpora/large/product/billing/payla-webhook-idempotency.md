---
name: payla-webhook-idempotency
description: Payla webhooks deduplicated on event_id in payla_events, async processing, and a state lattice that absorbs out-of-order events
type: project
status: active
verified: 2026-04-02
---

Payla livre les webhooks au moins une fois, sans garantie d'ordre. Deux événements sur trois arrivent en double pendant les pics (mesuré le 2026-01-05 : 38 % de doublons sur 21 400 événements). Le traitement doit donc être idempotent et tolérant à l'ordre. Le contexte général est dans [[payla-integration-overview]].

## Déduplication

Table `payla_events` : `event_id` (clé primaire, fourni par Payla), `type`, `received_at`, `processed_at`, `payload jsonb`. Le contrôleur fait un `INSERT ... ON CONFLICT (event_id) DO NOTHING` et répond `200` dans tous les cas, y compris quand l'insertion n'a rien fait. Répondre autre chose que `200` fait rejouer l'événement par Payla toutes les 10 minutes pendant 24 heures, ce qui a rempli les logs pendant l'incident [[payla-outage-2026-02]].

Le traitement est asynchrone : `PaylaEventProcessor` consomme les lignes avec `processed_at IS NULL` par lots de 500, toutes les 5 secondes, avec `FOR UPDATE SKIP LOCKED`. Un événement en échec reste `processed_at IS NULL` avec `attempts` incrémenté ; au-delà de 10 tentatives il passe en `dead = true` et l'astreinte reçoit l'alerte `payla_events_dead > 0`.

## Ordre des événements

Le cas courant : `debit.settled` reçu avant `debit.submitted` pour le même `debit_id`, parce que la soumission est confirmée par un autre nœud Payla. Au début (HF-2050) le processeur refusait la transition `pending -> settled` sans passer par `submitted`, et 1 200 prélèvements sont restés bloqués en `pending` la première semaine.

Depuis HF-2071 la machine d'état de `payments.status` est un treillis, pas une séquence : chaque événement porte un rang (`created` 0, `submitted` 1, `settled` 2, `failed` 2, `refunded` 3) et on n'applique que les transitions qui montent. Un `submitted` reçu après `settled` est absorbé sans erreur et journalisé en `debug`.

Deux états de même rang (`settled` et `failed`) ne peuvent pas se succéder ; si ça arrive, on écrit une ligne dans `payment_conflicts` et on laisse la réconciliation trancher le lendemain.

## Fenêtre de rejeu

On garde `payla_events` 90 jours (TTL par le job `PurgePaylaEvents` du dimanche 03:00). La fenêtre de rejeu de Payla est de 24 h, donc 90 jours suffisent largement pour la déduplication ; les 90 jours servent au support, qui cherche souvent « qu'est-ce que Payla nous a envoyé pour ce paiement ».

## Test de non-régression

`PaylaEventOrderingTest` rejoue les 6 permutations des trois événements `submitted`, `settled`, `refunded` et vérifie que l'état final est `refunded` et que `payments.settled_at` est renseigné dans les 6 cas. Ce test a détecté une régression le 2026-03-20 quand quelqu'un a réintroduit un `require(previous == SUBMITTED)`.
