---
name: replay-procedure-runbook
description: Procédure de rejeu d'un consommateur torrent, par horodatage ou par offset, avec la liste de contrôle (idempotence, effets de bord, capacité, annonce), la commande de réinitialisation d'offset, le mode « topic de rejeu » pour les cas partiels, et les quatre rejeux faits depuis 2025
type: reference
status: active
verified: 2026-06-24
---

# Rejouer un consommateur

Un rejeu, c'est faire relire à un groupe de consommateurs des messages qu'il a déjà consommés, en remettant son offset en arrière. Le topic les a encore (rétention 30 jours sur `domain.*`, compaction sur `cdc.*`, 400 jours sur `billing.*`, voir [[retention-and-compaction-policy]]). C'est l'opération la plus puissante et la plus dangereuse du bus, et c'est pourquoi elle a une liste de contrôle.

## Quand rejouer

- Un consommateur a eu un bug qui a produit une projection fausse (le cas de [[replay-2026-03-invoice-projection]]).

- Un consommateur a perdu sa cible (une table de l'entrepôt restaurée à un point antérieur, un index de recherche reconstruit).

- Un nouveau consommateur doit se construire un état à partir de l'historique (rejeu depuis le début du topic, ou depuis un instantané plus l'offset correspondant).

Pas quand : un consommateur a sauté un message qui a fini en file de rebut ([[dead-letter-topics-convention]]), c'est `reproduce` qui fait ça, sans toucher l'offset.

## La liste de contrôle

À remplir dans le ticket avant la commande. Chaque ligne a coûté quelque chose à quelqu'un.

1. **Idempotence.** Le consommateur peut-il relire un message déjà traité sans effet ? Une projection qui fait `INSERT` sans clé unique produit des doublons ; une qui fait `UPSERT` sur l'identifiant de l'agrégat va bien. `ingest-svc` a sa table d'offsets et déduplique. `notify-fanout` **n'est pas idempotent au sens utile** : rejouer `domain.load.assigned` renvoie les notifications, même si la clé d'idempotence côté pipeline de notifications les bloque… seulement pendant 180 jours et seulement pour le même événement métier. Un rejeu sur `notify-fanout` se fait avec la variable `NOTIFY_FANOUT_DRY_RUN=1`, ou pas du tout.

2. **Effets de bord.** Le consommateur appelle-t-il un service externe ? `billing-svc` déclenche des virements Payla sur `domain.bid.accepted` : un rejeu de `billing-svc` s'arrête à la frontière de Payla par le drapeau `BILLING_REPLAY_MODE=1`, qui écrit ce qu'il aurait fait dans `billing.replay_log` au lieu de le faire.

3. **Compaction.** Sur un `cdc.*` compacté, l'historique intermédiaire n'existe plus : rejouer donne le dernier état de chaque clé, pas la séquence. Si le consommateur a besoin de la séquence (un historique de statuts), c'est `cdc.app.load_status_history` qui la porte, pas `cdc.app.loads`.

4. **Capacité.** Combien de messages, à quel débit le consommateur rattrape-t-il, et qu'est-ce qu'il sature en aval ? Un rejeu de 30 jours de `domain.load.status-changed` (2,8 milliards de messages) vers l'entrepôt à 3× le débit normal prend 10 jours et sature les insertions ; on le fait par tranche de 3 jours, la nuit. Le panneau « catch-up rate » de [[consumer-lag-alerting]] donne le débit de rattrapage réel.

5. **Alertes.** Le retard va exploser, c'est voulu : poser un silence sur `ConsumerLagHigh{group}` pour la durée prévue, avec le ticket en commentaire. Pas de silence sur `torrent_partition_lag_stuck` : une partition bloquée pendant un rejeu est un vrai problème.

6. **Annonce.** Les propriétaires des projections en aval et le support sont prévenus, avec la fenêtre et ce qu'ils verront (données figées, puis rattrapage).

7. **Point de retour.** Noter les offsets courants du groupe avant de les toucher (`torrent-consumer-groups --describe --group <g>` dans le ticket). C'est la seule façon de revenir en arrière si le rejeu était une mauvaise idée.

## La commande

Le groupe doit être arrêté (aucun membre) : `torrent-consumer-groups` refuse sinon. Puis, depuis `ops-tools`, avec le fichier d'administration :

```
torrent-consumer-groups --bootstrap-server torrent.hf.internal:9093 \
  --command-config /etc/torrent/admin.properties \
  --reset-offsets --group billing-projector \
  --topic billing.invoice.lifecycle \
  --to-datetime 2026-03-02T00:00:00.000 --dry-run
```

Lire la sortie (un offset par partition), la coller dans le ticket, remplacer `--dry-run` par `--execute`, redémarrer le consommateur. `--to-datetime` cherche le premier message dont l'horodatage est supérieur ou égal : avec `CreateTime` posé par le producteur, c'est l'heure de production. Prendre une marge d'une heure en arrière : le rejeu d'un message déjà bien traité coûte un `UPSERT`, le manque d'un message coûte une nouvelle enquête.

Autres formes : `--to-offset` par partition (`--topic t:17`), `--shift-by -1000`, `--to-earliest`, `--by-duration PT2H`.

## Le mode « topic de rejeu »

Quand on ne veut rejouer qu'un sous-ensemble (les messages d'un seul transporteur, ceux d'une plage de clés), remettre l'offset du groupe ferait relire tout. À la place : un job (`torrent-replay-extract`) lit le topic source sur la plage, filtre, et écrit les messages retenus dans `replay.<groupe>.<ticket>` (famille `replay`, rétention 7 jours, créé et supprimé par le job). Le consommateur est lancé une fois avec `--topics replay.<groupe>.<ticket>` en plus de ses topics normaux, avec un `group.id` temporaire, puis le topic est supprimé. C'est ainsi qu'a été fait le rejeu des factures de mars, sur 3 % des messages au lieu de 100 %.

## Historique des rejeux

| Date | Groupe | Portée | Durée | Raison |
|---|---|---|---|---|
| 2025-09 | `search-indexer` | `cdc.app.loads` depuis le début | 6 h | reconstruction de l'index après changement de mapping |
| 2025-12 | `ingest-svc` | 3 topics, 2 jours | 5 h | table `raw.bids` restaurée après une erreur de migration |
| 2026-03 | `billing-projector` | topic de rejeu, 14 jours filtrés | 40 min | [[replay-2026-03-invoice-projection]] |
| 2026-06 | `eta-projector` | `cdc.app.loads` 7 jours | 2 h | correction d'un bug de fuseau horaire dans la projection |

Aucun n'a produit de doublon, aucun n'a envoyé de notification ou de virement, parce que la liste a été suivie. Le rejeu de décembre a rappelé que `ingest-svc` avait besoin de 900 s de tolérance de retard (voir la note de lag).
