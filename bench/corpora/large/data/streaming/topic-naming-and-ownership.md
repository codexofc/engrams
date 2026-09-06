---
name: topic-naming-and-ownership
description: Un topic s'appelle <famille>.<domaine>.<sujet>, déclaré dans topics.yaml (propriétaire, clé, rétention, partitions, schéma), créé par la CI, jamais à la main
type: reference
status: active
verified: 2026-04-16
---

## Le nom

`<famille>.<domaine>.<sujet>`, en minuscules, points comme séparateurs, tirets à l'intérieur d'un segment si besoin. Les familles sont fermées : `cdc`, `domain`, `billing`, `driver`, `ml`, `ops`, plus `dlq` pour les files de rebut ([[dead-letter-topics-convention]]) et `replay` pour les topics temporaires de rejeu. Une nouvelle famille est une décision d'équipe, pas un nom dans une MR.

- `cdc.app.loads` : la famille dit la nature (capture de changements), le domaine dit la source (`app` est la base PostgreSQL de l'API), le sujet est la table.

- `domain.load.assigned` : un événement métier, au passé, nommé par l'agrégat puis le fait.

- `driver.positions` : un flux, au pluriel quand c'est une série de mesures.

- `ops.metrics.notifications-daily` : le tiret dans le dernier segment est admis.

Pas de numéro de version dans le nom (`domain.load.assigned.v2`) : la compatibilité des schémas est gérée par le registre ([[schema-registry-compatibility-rules]]), et un changement incompatible crée un nouveau sujet avec un nom qui dit quoi (`domain.load.assignment-changed`), pas un suffixe.

## La déclaration

Tout topic existe d'abord dans `data-platform/torrent/topics.yaml` :

```
- name: domain.bid.placed
  owner: product-pricing
  key: bid_id
  partitions: 24
  retention: 30d
  cleanup: delete
  schema: domain.bid.placed
  consumers_expected: [ingest-svc, pricing-projector, notify-fanout]
  description: une enchère posée par un transporteur, une par version d'enchère
```

La CI du dépôt valide le fichier (nom conforme, propriétaire existant dans la liste des équipes, schéma présent dans le registre, partitions dans la liste autorisée) puis applique la différence avec le cluster via `torrent-topics --alter` ou `--create`. Une suppression demande le champ `deleted: <date>` et une approbation de deux personnes de l'équipe données ; le topic est vidé (rétention 1 h) pendant une semaine avant d'être supprimé, pour laisser le temps à un consommateur oublié de crier.

`auto.create.topics.enable = false` depuis mars 2025, après qu'un client de test a créé `test-loads-2` en production avec 1 partition et une rétention par défaut de 7 jours. Un producteur qui écrit vers un topic inexistant reçoit une erreur, ce qui est le comportement voulu.

## Le propriétaire

Le champ `owner` est une équipe, jamais une personne. Il décide du schéma, de la rétention, du nombre de partitions ([[partition-count-decisions]]), reçoit l'alerte de retard de ses consommateurs ([[consumer-lag-alerting]]) et répond de ce que contient le topic. Pour les `cdc.*` le propriétaire est l'équipe qui possède la table source, pas l'équipe données : si `cdc.app.loads` contient une colonne qui ne devrait pas y être, c'est le produit qui décide.

`consumers_expected` est informatif mais vérifié : un groupe de consommateurs qui lit un topic sans y être listé déclenche un avertissement hebdomadaire (`ops.torrent.unlisted-consumers`), et la revue mensuelle des topics le règle en ajoutant le groupe ou en demandant à son propriétaire ce qu'il fait là. Deux fois depuis 2025 la réponse était « je ne savais pas que ce service lisait encore ça » et le consommateur a été éteint.

## Les ACL

Une application par certificat, un certificat par application, et les ACL suivent `topics.yaml` : le propriétaire écrit, les `consumers_expected` lisent, personne d'autre. La CI génère les ACL à partir du fichier. Un nouveau consommateur commence donc par une MR sur `topics.yaml`, ce qui fait que la question « qui lit ce topic » a une réponse dans Git avant d'en avoir une dans le cluster.

## Ce qui reste manuel

Le compte de partitions ne diminue jamais (le protocole ne le permet pas) et l'augmenter change la répartition des clés : c'est une opération planifiée avec le propriétaire, décrite dans la note sur les partitions, et la CI la refuse sans le champ `partitions_changed: <date>` dans la déclaration.
