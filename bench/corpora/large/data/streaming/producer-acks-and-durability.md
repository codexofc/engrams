---
name: producer-acks-and-durability
description: Tous les producteurs métier écrivent en acks=all, idempotents, via une table outbox et un relais; driver.positions en acks=1 sans outbox, et ce que ça garantit
type: reference
status: active
verified: 2026-05-28
---

## Ce que « écrit » veut dire

Un message est considéré écrit quand le broker meneur de la partition et au moins un réplica (`min.insync.replicas = 2` sur un facteur de réplication 3) l'ont sur disque, et que le producteur a reçu l'accusé (`acks=all`). Un broker qui tombe après ça ne perd rien ([[incident-2026-05-torrent-3-disk-full]] : 0 message perdu). Deux brokers qui tombent en même temps sur la même partition rendent la partition indisponible en écriture (pas assez de réplicas synchrones), ce qui est préférable à écrire sur un seul et perdre à la panne suivante. C'est le réglage de tous les topics `domain.*`, `billing.*`, `cdc.*` et `ml.*`.

Le producteur est idempotent (`enable.idempotence = true`) : une nouvelle tentative après un accusé perdu ne duplique pas le message dans la partition. Ça protège du doublon réseau, pas du doublon applicatif (deux appels métier qui produisent deux messages), qui est l'affaire de la clé d'idempotence côté consommateur ([[exactly-once-vs-idempotent-consumers]]).

## L'outbox

L'API et l'outil dispatch ne produisent pas vers torrent depuis la transaction métier. Ils écrivent l'événement dans la table `outbox(id, topic, key, headers, payload, created_at, published_at NULL)` dans la même transaction PostgreSQL que la modification métier. Un relais (`outbox-relay`, 2 instances, une active par bail) lit `outbox` par lots de 500 dans l'ordre de `id`, produit vers torrent avec `acks=all`, et pose `published_at`. Une ligne dont la production échoue est retentée indéfiniment ; le relais ne passe pas à la suivante de la même clé tant que la précédente n'est pas publiée, ce qui préserve l'ordre par clé.

Pourquoi : sans outbox, la transaction métier valide puis la production échoue, et l'événement n'existe pas alors que le fait existe ; ou la production réussit puis la transaction est annulée, et l'événement dit quelque chose de faux. Les deux sont arrivés en 2024. Avec l'outbox, l'événement est publié « au moins une fois, éventuellement en retard, jamais faux ». Le retard médian entre `created_at` et `published_at` est de 120 ms, le p99 de 2 s, alerte `warn` à 30 s sur la plus vieille ligne non publiée, `page` à 5 minutes.

Les lignes publiées sont purgées à 48 h. La table a 20 000 lignes non publiées au pire du pic du matin.

## L'exception : `driver.positions`

Le backend mobile produit les positions directement, `acks=1` (le meneur seul), sans outbox, sans idempotence. Une position perdue sur 9 000 par seconde n'a pas d'importance : la suivante arrive 10 secondes plus tard. Le coût de `acks=all` sur ce flux serait 40 % de latence de production en plus au pic, mesuré en 2025, pour une garantie dont personne n'a l'usage. Le topic reste en facteur de réplication 3 (pour la lecture et pour la panne de broker), mais l'écriture ne les attend pas. `driver.state-reported`, lui, est en `acks=all` via l'outbox du backend mobile, parce qu'un rapport de tachygraphe manqué a un effet sur l'ETA.

## Les en-têtes que tout producteur pose

Voir [[event-envelope-format]]. Le relais les copie depuis `outbox.headers` sans les interpréter, sauf `produced-at` qu'il pose lui-même.

## Réglages côté producteur

| Paramètre | Valeur | Raison |
|---|---|---|
| `acks` | `all` (sauf positions) | ci-dessus |
| `enable.idempotence` | `true` | ci-dessus |
| `max.in.flight.requests.per.connection` | 5 | le maximum compatible avec l'idempotence et l'ordre |
| `linger.ms` | 20 | regrouper sans que ça se voie ; le relais envoie par lots de toute façon |
| `compression.type` | `zstd` | JSON compressé à 1/6, CPU négligeable |
| `delivery.timeout.ms` | 120 000 | deux minutes de tentatives avant que le relais considère l'envoi échoué et le retente à son propre rythme |
| `request.timeout.ms` | 30 000 | |

`compression.type` est décidé par le producteur, et un topic mélange donc des lots compressés différemment si un producteur oublie. La CI des clients wrappers impose `zstd` ; le seul producteur hors wrappers est un outil de test.

## Ce que ça ne garantit pas

- L'ordre entre clés différentes : deux chargements distincts sont sur deux partitions, leurs événements arrivent dans un ordre quelconque l'un par rapport à l'autre.

- L'ordre entre topics : `domain.bid.accepted` et `cdc.app.bids` pour la même enchère partent de deux chemins (outbox et slot de réplication) et un consommateur qui lit les deux les voit dans un ordre non défini. Le projecteur qui a besoin des deux attend d'avoir les deux, il ne suppose pas.

- Qu'un message produit sera consommé : c'est l'affaire du consommateur, de son retard ([[consumer-lag-alerting]]) et de sa file de rebut.
