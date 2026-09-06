---
name: webhook-delivery-outbox
description: Outgoing webhooks go through the sys_outbox table written in the same transaction as the domain change, relayed by a LISTEN/NOTIFY worker with at-least-once delivery, HMAC-SHA256 signature and 7 retries over 24 h
type: project
status: active
verified: 2026-05-14
---

# Webhooks sortants : outbox transactionnelle

Livrés en HF-1240 (janvier 2026) pour remplacer l'envoi direct depuis les handlers Messenger, qui perdait des événements quand la transaction était rollbackée après l'envoi, ou en envoyait pour des changements jamais commités. On avait 0,4 % d'événements fantômes mesurés sur décembre 2025 en comparant `load_events` et les logs de livraison.

## Principe

1. Le code métier fait son changement et, dans la même transaction, insère une ligne dans `sys_outbox`. Il n'appelle jamais HTTP.
2. Un trigger `AFTER INSERT` sur `sys_outbox` fait `NOTIFY outbox_new`.
3. Un worker (`app:outbox:relay`, un seul pod, `DATABASE_URL_LISTEN` en direct sans PgBouncer) écoute, lit les lignes `status = 'PENDING'` par lot de 100 avec `FOR UPDATE SKIP LOCKED`, les envoie, et met à jour le statut.
4. Un CronJob toutes les minutes lance le même relay en mode `--sweep` pour rattraper ce que le `NOTIFY` aurait manqué (redémarrage du worker, notification perdue).

## Table

```sql
CREATE TABLE sys_outbox (
  id uuid PRIMARY KEY,               -- v7, donc trié
  subscription_id uuid NOT NULL,
  event_type text NOT NULL,          -- 'load.dispatched', 'bid.accepted', ...
  payload jsonb NOT NULL,
  schema_version smallint NOT NULL DEFAULT 2,
  status text NOT NULL DEFAULT 'PENDING',  -- PENDING | DELIVERED | FAILED | DEAD
  attempts smallint NOT NULL DEFAULT 0,
  next_attempt_at timestamptz NOT NULL DEFAULT now(),
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  delivered_at timestamptz
);
CREATE INDEX idx_sys_outbox_pending ON sys_outbox (next_attempt_at) WHERE status IN ('PENDING','FAILED');
```

Une ligne par abonnement et par événement. Un chargeur avec trois webhooks configurés produit trois lignes pour un `load.dispatched`.

## Livraison

`POST` vers l'URL de l'abonnement, corps = `payload`, en-têtes :

- `X-Halden-Event: load.dispatched`
- `X-Halden-Delivery: <id de la ligne>` (le destinataire déduplique là-dessus)
- `X-Halden-Timestamp: <unix seconds>`
- `X-Halden-Signature: sha256=<hex>` = HMAC-SHA256 du `<timestamp>.<corps>` avec le secret de l'abonnement (32 octets aléatoires, montré une fois à la création, stocké chiffré avec `sodium_crypto_secretbox` et la clé `APP_WEBHOOK_SECRET_KEY`).

Timeout 10 s. Un 2xx = `DELIVERED`. Autre chose = `FAILED`, `attempts + 1`, `next_attempt_at = now() + backoff`. Backoff : 30 s, 2 min, 10 min, 30 min, 1 h, 4 h, 12 h. Après 7 tentatives (environ 18 h), `DEAD` et un e-mail à l'admin de l'organisation. Un abonnement avec 50 `DEAD` sur 7 jours est désactivé automatiquement (`subscriptions.disabled_reason = 'too_many_failures'`).

Un 410 Gone désactive l'abonnement immédiatement, c'est documenté pour les intégrateurs.

## Garanties

At-least-once. Le pire cas : le worker envoie, le destinataire répond 200, le worker meurt avant l'`UPDATE`. La ligne est renvoyée au sweep suivant. D'où le `X-Halden-Delivery` pour dédupliquer côté destinataire. Aucune garantie d'ordre entre événements, chaque payload contient `occurred_at` et le destinataire trie s'il en a besoin.

## Rétention

`DELIVERED` conservés 14 jours, `DEAD` 90 jours (le support s'en sert), purge par `app:outbox:purge` la nuit. La table fait 2,1 Go en régime de croisière. Pas partitionnée, la purge par lots de 10 000 suffit.

## Payload et version

`schema_version` est dans la table et dans le corps JSON. On est en version 2. Un changement de version est un nouveau champ dans la table, pas une réécriture : les abonnements ont un `payload_version` et le relay sérialise selon cette version. Deux versions coexistent depuis mars 2026 et il reste 4 abonnements en version 1.

## Ce qu'on a mesuré

Latence entre commit et livraison : p50 180 ms, p95 900 ms grâce au `NOTIFY`. Sans lui (sweep seul) on serait à 30 s de moyenne. Débit : 12 000 livraisons par jour en semaine, pic à 40 par seconde à 7 h 30.

## Pièges rencontrés

- Le trigger `NOTIFY` dans une transaction longue ne part qu'au commit, ce qui est ce qu'on veut, mais la première version du worker faisait un `SELECT` immédiatement à la réception et parfois ne voyait rien : la notification est émise au commit, mais la visibilité de la ligne pour une autre connexion peut suivre de quelques millisecondes. Le worker attend maintenant 50 ms après une notification avant de lire.
- `SKIP LOCKED` avec deux relays en parallèle fonctionne, mais un seul pod suffit et deux pods compliquent le débogage. Un seul pod, `strategy: Recreate`.
- Voir [[messenger-transports-and-retries]] pour la distinction avec les messages internes : l'outbox ne sert qu'aux clients externes.
