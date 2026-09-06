---
name: streaming-team-preferences
description: Habitudes de l'équipe données sur le bus: tout dans topics.yaml, un propriétaire par topic, des secondes pas des messages, consommateurs idempotents
type: user
status: active
verified: 2026-06-24
---

Habitudes de l'équipe données (quatre personnes, dont deux sur le bus) pour torrent, telles qu'appliquées en 2026.

- **Git avant le cluster.** Un topic, un schéma, une ACL, un groupe de consommateurs existent d'abord dans `topics.yaml` ou dans `schemas/` ([[topic-naming-and-ownership]]). La commande `torrent-topics --create` à la main est réservée aux topics `replay.*` par l'outil d'extraction, et même ceux-là sont nommés par le ticket.

- **Un propriétaire par topic, une équipe, pas une personne.** Un topic orphelin est supprimé après la procédure d'une semaine.

- **Des secondes, pas des messages.** Le retard se mesure en temps derrière la tête du journal ([[consumer-lag-alerting]]). Quelqu'un qui dit « on a 50 000 messages de retard » se fait demander « sur quel topic, et ça fait combien de secondes ».

- **Un consommateur est idempotent, ou il a un mode rejeu explicite, ou il n'est pas critique.** Les trois questions de la revue ([[exactly-once-vs-idempotent-consumers]]) sont posées avant la fusion, pas après le premier rejeu.

- **La file de rebut n'est pas une option.** Depuis février 2026, un consommateur sans file de rebut porte un argument nommé qui dit pourquoi, et la revue lit la raison.

- **Le schéma dit l'unité.** `distance_km`, `amount_cents`, ou `x-unit`. Un nombre sans unité ne passe pas la CI.

- **Pas de fonctionnalité du broker qu'on n'a pas lue.** Transactions, stockage hiérarchisé, quotas : on a lu le code ou la spécification, testé en staging, et écrit pourquoi on ne l'utilise pas. Les transactions ont été utilisées puis retirées ; le stockage hiérarchisé attend un besoin.

- **Le rejeu est une procédure avec une liste, jamais une commande.** Les sept points de [[replay-procedure-runbook]] sont remplis dans le ticket avant `--execute`. Quatre rejeux en deux ans, zéro doublon, zéro notification envoyée deux fois.

- **L'astreinte plateforme a le broker, nous avons les consommateurs.** La nuit, un broker qui tombe est un problème de machine et la plateforme sait le redémarrer ; un consommateur en retard attend le matin sauf les neuf groupes critiques, qui appellent leur propriétaire.

- **Français ou anglais** dans les notes et les tickets, selon qui écrit. Noms de topics, de champs, de groupes en anglais.

- **Revue mensuelle des topics**, le premier mardi : consommateurs non déclarés, sujets sans producteur, champs dépréciés en retard, disque par broker, et la liste des files de rebut non revues. Une heure, un compte rendu de dix lignes.
