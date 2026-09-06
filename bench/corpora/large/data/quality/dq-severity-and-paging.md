---
name: dq-severity-and-paging
description: Trois sévérités (page, warn, info), page réservée aux invariants dont la violation fausse une décision ou un paiement, routage vers le propriétaire du domaine, regroupement par domaine et par 15 minutes, silence par ticket, et les 41 règles page relues chaque trimestre
type: reference
status: active
verified: 2026-06-26
---

## Les trois niveaux

| Sévérité | Ce que ça déclenche | Critère pour l'attribuer |
|---|---|---|
| `page` | appel du propriétaire du domaine, jour et nuit | un invariant dont la violation fausse une décision opérationnelle, un paiement ou un chiffre communiqué dans les 24 h |
| `warn` | message dans le canal du domaine, regroupé par 15 minutes | un problème réel qui peut attendre le matin ou la semaine |
| `info` | tableau de bord seulement | mesure de bruit de fond, règle en observation, ou candidate à la suppression |

Le critère de `page` est volontairement étroit. Une règle `page` qui appelle pour quelque chose qui peut attendre le matin est rétrogradée à la revue suivante, parce que la troisième fausse alerte nocturne est celle après laquelle l'astreinte cesse de lire les alertes. Les 41 règles `page` de juin 2026 ([[dq-framework-overview]]) sont relues chaque trimestre, une par une, avec la question « la dernière fois qu'elle a appelé, fallait-il se lever ». Réponses en mars 2026 : 3 oui, 0 non, 38 « jamais appelé ».

## Routage

Le champ `owner` de la règle ([[dq-rule-catalog-core]]) est un domaine : `dispatch`, `pricing`, `billing`, `ml`, `product`, `data`. Chaque domaine a une astreinte (ou, pour `product` et `ml`, une personne de garde en heures ouvrées et l'astreinte données la nuit). L'alerte va au domaine, pas à l'équipe données : c'est le propriétaire de la table qui sait si 38 chargements sans transporteur sont graves, et c'est lui qui a le runbook. L'équipe données reçoit une copie en `info` de tout, sur son tableau de bord.

Une règle `page` sans `runbook` est refusée par `dq-runner lint`. Le runbook est un fichier markdown court dans `dq/runbooks/` : quoi regarder d'abord (souvent la requête sur `sample_keys`), qui prévenir, et ce qu'il ne faut pas faire (« ne pas supprimer les doublons à la main avant d'avoir compris d'où ils viennent »).

## Regroupement

Les `warn` d'un même domaine sont regroupés par tranche de 15 minutes en un seul message listant les règles, pour qu'une panne d'ingestion qui fait échouer 20 règles de fraîcheur ([[freshness-monitors]]) produise un message et pas vingt. Les `page` ne sont pas regroupés entre règles mais dédoublonnés : une règle en échec continu appelle une fois, puis rappelle toutes les 4 heures si l'échec persiste et que le silence n'a pas été posé.

## Silence

`dq-runner silence <règle> --until <date> --ticket HF-xxxx --reason "..."` pose un silence qui apparaît sur le tableau de bord avec son ticket. Pas de silence sans ticket, pas de silence de plus de 7 jours sans renouvellement explicite. Les silences actifs sont la première ligne du compte rendu de la revue mensuelle ([[dq-rules-review-feedback]]) ; un silence renouvelé trois fois est une règle à changer ou un problème à régler, pas un silence à renouveler.

Les rejeux planifiés côté bus posent un silence sur les règles de fraîcheur et de réconciliation des tables concernées, avec le ticket du rejeu. C'est dans la liste de contrôle du rejeu.

## Ce qu'une alerte contient

- Le nom de la règle, sa description (la phrase « pourquoi ça compte » écrite par le propriétaire), la sévérité.

- `observed` contre `threshold` : « 38 lignes, seuil 0 », « 340 s, seuil 300 s », « écart 41 200 centimes, seuil 100 ».

- Les `sample_keys` (jusqu'à 20 clés primaires) pour les règles de ligne, ou le lien vers la requête enregistrée pour les réconciliations et les anomalies.

- Le lien du runbook et le lien du panneau de la règle sur 7 jours.

- Depuis février 2026, le dernier changement du modèle marmot de la table (commit et auteur), parce que la moitié des échecs suivent un déploiement.

## Erreurs d'exécution

Une règle dont la requête échoue (table absente, colonne renommée, délai dépassé) est en statut `error`, et une `error` est traitée comme un `warn` pour l'équipe données quelle que soit la sévérité de la règle : une règle qui ne tourne pas ne protège pas, et c'est presque toujours une migration de schéma qui n'a pas mis à jour la règle. Le moniteur de schéma ([[schema-drift-monitor]]) attrape la plupart de ces cas avant que la règle ne casse.

## Ce qu'on ne fait pas

- Pas de sévérité calculée automatiquement selon la taille de l'écart. Un écart de 1 ligne sur une clé primaire est un `page` ; un écart de 10 % sur un volume est un `warn`. La sévérité dit ce que la violation signifie, pas combien elle est grande.

- Pas de page vers l'équipe données pour une règle d'un domaine. La tentation existe (« ils sauront mieux »), le résultat serait une équipe données qui porte toutes les astreintes et des domaines qui ne lisent plus leurs règles.
