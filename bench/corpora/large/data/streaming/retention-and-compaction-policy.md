---
name: retention-and-compaction-policy
description: Rétention par famille de topics, 7 jours driver, 30 jours domain, 400 jours billing, compaction sur cdc avec tombstones à 7 jours, segments de 1 GB, tout changement passe par topics.yaml et la vérification par broker de la CI depuis mai 2026
type: reference
status: active
verified: 2026-06-05
---

## Les durées

| Famille | `cleanup.policy` | Rétention | Pourquoi |
|---|---|---|---|
| `driver.*` | `delete` | 7 jours | le volume ; l'entrepôt a 90 jours de positions, torrent n'est pas une archive |
| `domain.*` | `delete` | 30 jours | la fenêtre de rejeu utile ([[replay-procedure-runbook]]) : aucun rejeu n'a jamais remonté à plus de 14 jours |
| `billing.*` | `delete` | 400 jours | la réconciliation annuelle avec Payla relit l'année |
| `ml.*` | `delete` | 30 jours | comme `domain` |
| `cdc.app.*` | `compact` | pas de durée, tombstones 7 jours | l'état courant de chaque ligne, pour construire une projection |
| `ops.*` | `delete` | 7 jours, sauf `ops.dlq.decisions` 400 jours | interne |
| `dlq.*` | `delete` | 90 jours | plus long que la source, voir [[dead-letter-topics-convention]] |
| `replay.*` | `delete` | 7 jours | temporaire par construction |

La rétention est en temps (`retention.ms`), jamais en octets (`retention.bytes` reste à -1) : une rétention en octets par partition rend la durée réellement conservée dépendante du débit et différente d'une partition à l'autre, ce qui rend le « on a 30 jours » faux sans que personne le voie. On préfère savoir combien de temps on a et surveiller le disque.

## La compaction des `cdc.*`

`cleanup.policy = compact`, `min.cleanable.dirty.ratio = 0.3` (le nettoyeur passe quand 30 % du journal d'une partition est « sale », c'est-à-dire porte des clés qui ont une version plus récente), `segment.bytes = 1 GB`, `min.compaction.lag.ms = 1 h` (un message ne peut pas être compacté dans l'heure qui suit sa production, pour qu'un consommateur en léger retard voie encore la séquence), `delete.retention.ms = 7 jours` (un tombstone, le message à valeur nulle qui suit une suppression, reste 7 jours avant d'être lui-même retiré, pour qu'un consommateur en retard de moins de 7 jours voie la suppression).

Le segment actif n'est jamais compacté : avec des segments de 1 GB et un topic à faible débit comme `cdc.app.carriers` (15 changements par seconde), le segment actif peut contenir plusieurs jours, et l'état « compacté » est donc en réalité « compacté sauf les derniers jours ». C'est sans conséquence pour les consommateurs qui prennent le plus grand `lsn` par clé ([[cdc-tap-postgres-connector]]) et c'est pourquoi cette règle existe.

Taille après compaction, juin 2026 : `cdc.app.loads` 41 GB (3,2 M de chargements, 31 versions de schéma, `before` et `after` complets), `cdc.app.bids` 88 GB, `cdc.app.load_status_history` 120 GB (une table d'historique compactée sur sa clé primaire ne se compacte jamais vraiment, chaque ligne est unique ; on garde la compaction pour la sémantique et on accepte la taille). Les autres sous 10 GB.

## Changer une rétention

Uniquement par `topics.yaml` ([[topic-naming-and-ownership]]). Depuis mai 2026 ([[incident-2026-05-torrent-3-disk-full]]), la CI calcule la projection du disque par broker et refuse au-dessus de 70 %. Un allongement de rétention est aussi un choix de ce qu'on veut pouvoir rejouer : 30 jours sur `domain.*` a été retenu parce qu'aucun rejeu n'en a eu besoin de plus, et parce que 60 jours feraient 1,1 TB de plus après réplication pour une hypothèse.

Un raccourcissement prend effet en quelques minutes (le nettoyeur passe toutes les 5 minutes) et il est irréversible : les segments supprimés le sont. La CI demande une seconde approbation pour tout raccourcissement sur `billing.*`.

## Ce qu'on ne fait pas

- Pas de stockage hiérarchisé (segments anciens déchargés vers le stockage objet) : la fonction existe dans la version 3.4 du broker, on l'a testée en staging, elle marche, et on n'en a pas besoin à 55 % de disque avec une croissance de 40 GB par semaine. La décision est revue à chaque cycle matériel.

- Pas de rétention infinie sur `domain.*` « pour l'audit » : l'audit est dans l'entrepôt (`raw.*`, 400 jours) et dans PostgreSQL, qui sont faits pour être interrogés. Un topic n'est pas interrogeable, c'est un journal.

- Pas de rétention différente entre partitions d'un même topic : ça n'existe pas dans le protocole et c'est bien.
