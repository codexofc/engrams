---
name: shipper-bid-digest-batching
description: Depuis février 2026 les notifications d'enchère aux expéditeurs sont regroupées en digest horaire (immédiat pour la première enchère et les 2 h avant clôture), 71 % d'e-mails en moins sur ce type, HF-4150
type: project
status: active
verified: 2026-04-14
---

# Regrouper les notifications d'enchère aux expéditeurs

## Le problème

Un expéditeur qui publie un chargement reçoit un e-mail par enchère reçue (`bid.received`). Un chargement Paris-Milan attractif reçoit 30 à 60 enchères en deux heures. Les expéditeurs qui publient dix chargements par jour recevaient 300 e-mails. En janvier 2026, `bid.received` représentait 41 % des e-mails sortants et 52 % des plaintes ([[complaint-rate-and-feedback-loops]]). Trois expéditeurs parmi les vingt plus gros avaient simplement filtré `mail.halden.example` vers la corbeille, digests futurs compris.

## Ce qu'on a livré (HF-4150, 2026-02-09)

Un type d'événement `bid.digest`, e-mail seulement, et une règle de regroupement dans `event_types.yaml` :

```
bid.received:
  channels: [push, email]
  email:
    batch:
      window: 1h
      key: "{recipient_id}:{load_id}"
      immediate_if:
        - first_bid_on_load
        - closes_within: 2h
      digest_event: bid.digest
```

`BatchingDispatcher` intercepte les livraisons e-mail dont l'événement a une règle `batch`. La première enchère sur un chargement part tout de suite (l'expéditeur veut savoir que ça bouge). Les suivantes sont mises dans `notification_batches(key, recipient_id, opens_at, items JSONB)` et un message Messenger différé de `window` déclenche le rendu du digest, un e-mail par chargement et par heure au maximum, avec la liste des enchères (transporteur, prix, délai), triée par prix. Quand la clôture des enchères est à moins de 2 h, on repasse en immédiat parce que l'expéditeur est en train de choisir.

La cloche de l'application et le push ne sont pas regroupés : ils ne coûtent rien en réputation et le push est déjà limité par [[per-recipient-rate-limits]].

## Mesuré

Quatre semaines avant (janvier) contre quatre semaines après (mars 2026) :

| | Avant | Après |
|---|---|---|
| e-mails `bid.*` par semaine | 248 000 | 72 000 |
| plaintes sur `bid.*` par semaine | 38 | 9 |
| délai médian entre enchère et attribution | 3 h 40 | 3 h 35 |
| enchères attribuées dans l'heure | 22 % | 23 % |
| ouvertures (clic sur le lien `?ref=`) par e-mail envoyé | 18 % | 44 % |

Le délai d'attribution n'a pas bougé, ce qui était la crainte du produit (« l'expéditeur attribue quand il reçoit l'e-mail »). En réalité, l'attribution se fait dans l'outil, l'e-mail ne fait que ramener l'expéditeur dedans, et un digest le ramène aussi bien.

## Ce qui a coincé

- Le digest rendu à l'heure pile pour tous les expéditeurs a produit un pic de 4 000 e-mails à chaque heure ronde, et Courrix a ralenti les acceptations (429 doux) pendant deux jours. La fenêtre est maintenant décalée par destinataire : `opens_at = now + window + hash(recipient_id) % 600 s`. Le pic est étalé sur dix minutes.

- Une enchère retirée puis remplacée (le « changer mon prix » côté transporteur, qui est une nouvelle enchère) apparaissait deux fois dans le digest. Le rendu déduplique par transporteur et garde la dernière.

- Les expéditeurs qui avaient filtré notre domaine ne voient pas le digest non plus. Le support a contacté les trois gros ; deux ont retiré le filtre.

## Ce qu'on n'a pas fait

- Un digest quotidien en plus de l'horaire : personne ne l'a demandé après la mise en place, et deux niveaux de regroupement compliquent le rendu et les tests.

- Regrouper d'autres types d'événements. `load.status_changed` pour les gros transporteurs a eu un digest push en mars, sur le même mécanisme, mais e-mail pour les autres types reste immédiat : `invoice.issued` ou `load.assigned` sont des événements que le destinataire attend un par un.

## Voir aussi

Le regroupement se fait après le `dispatch()` et avant la file, donc l'intention dans `notifications` reste une ligne par enchère, et `notifications:trace` sur une enchère regroupée montre `batched_into: <batch_id>` puis la livraison du digest. Le reste du chemin est celui de [[notifications-pipeline-overview]].
