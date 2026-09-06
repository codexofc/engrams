---
name: storage-oncall-preferences
description: Habitudes de l'astreinte stockage: une sauvegarde n'existe que restaurée, trois copies nommées, aucune suppression à la main, les gros écrits s'annoncent
type: user
status: active
verified: 2026-06-30
---

Habitudes des deux personnes qui tiennent l'astreinte stockage (une plateforme, une ops), telles qu'appliquées en 2026.

- **Une sauvegarde qui n'a pas été restaurée n'existe pas.** La colonne « dernière restauration réelle » de l'inventaire ([[backup-inventory-and-retention]]) est la seule qui compte. Une ligne à plus de douze mois passe en gras et devient un point de l'exercice suivant.

- **Trois copies, et on sait laquelle survit à quoi.** La question posée pour chaque donnée n'est pas « est-ce sauvegardé » mais « où sont les trois copies, et laquelle reste si le rack A brûle, si le datacentre brûle, si quelqu'un tape `rm` ». Les réponses sont dans l'inventaire, et l'exercice de mai a trouvé un bucket dont la réponse était fausse.

- **Personne ne supprime à la main dans un bucket de données.** Les clés d'application n'ont pas le droit, la clé d'administration non plus, et la clé de suppression se délivre pour 4 heures avec un ticket ([[restore-2025-11-documents-prefix-deleted]]). La règle vaut pour nous aussi, surtout pour nous.

- **Les gros écrits s'annoncent.** Au-dessus de 1 TB vers le stockage objet, un message dans le canal et une ligne dans `CHANGES.md`, quelle que soit l'équipe ([[incident-2026-03-appliance-replication-lag]]).

- **La capacité se décide dix mois avant.** Un tiroir de disques se commande en six semaines, un plan se fait à 75 %, pas à 90 % ([[storage-capacity-plan-2026]]). Le `warn` à 75 % qu'on silencie en connaissance de cause a une date de fin et un ticket.

- **Le chiffre du rejeu de WAL est 1,2 GB par 10 minutes.** Toute estimation de durée de restauration part de là. Il est remesuré à chaque exercice.

- **Les clés sur papier, dans des enveloppes, ouvertes à l'exercice.** Une clé qu'on n'a jamais lue depuis l'enveloppe est une clé dont on ne sait pas si elle est lisible ([[backup-encryption-and-key-custody]]).

- **L'exercice trimestriel n'est pas négociable.** Une journée, sur le cluster jetable, depuis les sauvegardes seules, avec les gens qui seraient là un vrai lundi matin. Les six constats de mai valent plus que six mois de tableaux de bord verts.

- **Une restauration sur la prod se fait à deux.** Une personne exécute, une relit chaque requête d'écriture avant. Posé en février, appliqué depuis.

- **Français ou anglais**, selon qui écrit. Noms de buckets, de clés et de commandes en anglais.

- **Pas de fonctionnalité de l'appliance qu'on n'a pas essayée sur staging.** La restauration par lot des versions était dans la documentation du vendeur depuis deux ans et personne ne l'avait lancée avant d'en avoir besoin un mercredi après-midi. C'est la dernière fois.
