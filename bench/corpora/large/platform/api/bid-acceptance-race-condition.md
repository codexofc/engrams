---
name: bid-acceptance-race-condition
description: Two carriers could both get their bid accepted on the same load, fixed in HF-1350 with SELECT FOR UPDATE on loads and a partial unique index on bids(load_id) WHERE status='ACCEPTED'
type: project
status: active
verified: 2026-01-22
---

# Double acceptation d'offres sur un même chargement

## Le symptôme

Le 4 décembre 2025, le support reçoit deux transporteurs qui affirment chacun avoir remporté le chargement `L-2025-118204`. Les deux ont reçu la notification push "Votre offre a été acceptée", les deux ont une ligne `bids` en `ACCEPTED`. Le chargement lui-même est en `DISPATCHED` avec `carrier_id` du second.

Sur la semaine, `SELECT load_id, count(*) FROM bids WHERE status = 'ACCEPTED' GROUP BY 1 HAVING count(*) > 1` remonte 7 cas. Aucun avant octobre. Ce qui a changé en octobre : l'acceptation automatique des offres au prix cible (`AutoAcceptBidHandler`, HF-1288), qui tourne en worker Messenger en même temps que l'acceptation manuelle par le chargeur dans l'API.

## La course

`BidAcceptanceService::accept(Bid $bid)` faisait, dans une transaction :

1. `$load = $bid->getLoad()` (déjà chargé, pas de lock)

2. `if ($load->getStatus() !== LoadStatus::BIDDING) throw`

3. `$bid->setStatus(ACCEPTED)`, `$load->setStatus(DISPATCHED)`, `$load->setCarrier(...)`

4. `flush()`

Deux transactions passent l'étape 2 en même temps, chacune écrit sa version. Sous `READ COMMITTED` (notre défaut), la seconde écrase la première sans erreur. Doctrine ne fait pas de version optimiste sans `#[Version]`, et `Load` n'en avait pas.

## Le correctif (HF-1350, déployé le 11 décembre 2025)

Deux couches, parce qu'une seule ne suffit pas.

**Verrou pessimiste sur le chargement.** `LoadRepository::findForUpdate(Uuid $id)` fait `SELECT ... FOR UPDATE NOWAIT`. `NOWAIT` plutôt qu'attendre : si un autre processus tient déjà le verrou, on répond 409 `load_locked` et le client réessaie. Le mobile réessaie trois fois avec 200 ms de jitter, le web affiche "Quelqu'un traite ce chargement". Le worker Messenger, lui, réessaie via le retry standard du transport.

Le `NOWAIT` a été discuté. `FOR UPDATE` bloquant aurait été plus simple, mais avec PgBouncer en mode transaction on ne veut pas de connexions serveur qui attendent, voir [[postgres-connection-pool-pgbouncer]].

**Contrainte en base.**

```sql
CREATE UNIQUE INDEX CONCURRENTLY uniq_bids_accepted_per_load
  ON bids (load_id) WHERE status = 'ACCEPTED';
```

C'est l'index partiel qui rend le bug impossible, même si quelqu'un contourne le service. La violation sort en `UniqueConstraintViolationException`, mappée en 409 `bid_already_accepted` par `ExceptionEnvelopeMapper` (voir [[api-error-envelope-convention]]).

Avant de créer l'index, il a fallu nettoyer les 7 doublons. Règle choisie avec le produit : la première offre acceptée (par `accepted_at`) reste, l'autre repasse en `REJECTED` avec `rejection_reason = 'duplicate_acceptance_cleanup'` et un e-mail d'excuse envoyé à la main par le support.

## Tests

`BidAcceptanceConcurrencyTest` lance deux processus PHP via `Symfony\Component\Process` qui acceptent deux offres du même chargement avec un `sleep(1)` entre lecture et écriture. Le test vérifie qu'exactement un des deux obtient 200. Il est lent (2 s) et tagué `@group concurrency`, exécuté seulement dans le job CI `tests-slow`. On l'a gardé parce qu'un test unitaire ne peut pas reproduire la course.

## Ce qu'on en retient

La machine à états de `Load` (voir [[load-status-state-machine]]) vérifie les transitions, mais une vérification en mémoire ne protège rien contre la concurrence. Toute transition qui a un effet métier exclusif (une seule offre gagnante, une seule facture par chargement) doit avoir sa contrainte en base.
