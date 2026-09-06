---
name: legacy-nfs-document-share
description: Jusqu'en 2024 les documents vivaient sur un partage NFS monté par les pods de l'API, rsync nocturne, sans versionnage; remplacé par le stockage objet
type: reference
status: archived
superseded_by: [[object-store-buckets-and-layout]]
verified: 2025-09-20
---

# Le partage NFS des documents (archivé)

Description de ce qui précédait le stockage objet, pour lire les tickets d'avant 2024. Le dispositif actuel est dans [[object-store-buckets-and-layout]].

## Ce que c'était

Un serveur `hf-files-01` dans le rack A avec 24 TB en RAID-6, exportant `/srv/documents` en NFS v4 vers les nœuds du cluster. Les pods de l'API montaient le partage (`ReadWriteMany`) et écrivaient les fichiers uploadés par les clients dans `/srv/documents/<load_id>/<document_id>.<ext>`. Le téléchargement passait par l'API, qui lisait le fichier et le renvoyait dans la réponse HTTP.

Sauvegarde : un `rsync` nocturne vers `hf-files-02` dans le rack B, puis, à partir de 2023, une copie hebdomadaire sur bande par le prestataire du datacentre, l'ancêtre du contrat actuel ([[offsite-weekly-copy-contract]]).

## Ce qui n'allait pas

- **Pas de versionnage, pas de corbeille.** Un fichier écrasé ou supprimé était perdu jusqu'au `rsync` de la nuit, et perdu tout court si le `rsync` était passé entre-temps (il propageait les suppressions avec `--delete`). Deux pertes documentées en 2023, 40 documents, retrouvés pour moitié sur les bandes.

- **Le téléchargement passait par l'API.** Un POD de 8 MB téléchargé par un client transitait par un pod PHP, ce qui bloquait un worker pendant la durée du transfert. Le pic du lundi matin (les expéditeurs qui vérifient les livraisons du week-end) saturait les workers et ralentissait tout le reste. La limite de taille d'upload de l'ingress était à 20 MB à cause de ça et le reste toujours, par habitude.

- **Le partage NFS et le cluster.** Un redémarrage de `hf-files-01` figeait tous les pods de l'API qui avaient un descripteur ouvert (`D state`, pas tuable), et le seul remède était de redémarrer les nœuds. Deux fois en 2023.

- **Une seule copie dans le rack pendant la journée.** Entre deux `rsync`, tout ce qui avait été écrit dans la journée n'existait qu'à un endroit.

- **Le chemin par `load_id`.** Un document rattaché par erreur au mauvais chargement était déplacé à la main sur le serveur par un opérateur, hors de toute transaction, et la base et le disque divergeaient. On a compté 1 200 fichiers orphelins à la migration.

- **Les droits.** Le partage était en lecture-écriture pour tout pod du cluster qui le montait. La séparation par environnement n'existait pas ; staging a écrit dans les documents de prod une fois, sur un préfixe de test, en 2022.

## La migration (2024)

Un bucket par environnement sur les appliances, l'upload par URL présignée depuis le client (le fichier ne touche plus l'API), le téléchargement pareil, la clé d'objet dérivée de `document_id` et de la date d'upload plutôt que du chargement, une colonne `sha256` dans `documents` vérifiée à l'upload. La copie des 6,2 TB existants a pris trois nuits avec un job qui lisait le NFS et écrivait dans le bucket en vérifiant le haché, puis une semaine de double lecture (l'API cherchait dans le bucket puis, à défaut, sur le NFS) avant de couper. Les 1 200 orphelins ont été mis dans un préfixe `orphans/` du bucket, revus par le support pendant six mois, 300 rattachés, le reste supprimé en 2025.

`hf-files-01` et `-02` ont été réaffectés comme nœuds de secours du cluster. Les bandes de 2023 sont chez le prestataire jusqu'à la fin de leur contrat de conservation, en 2028, et personne ne sait plus très bien si le lecteur pour les lire existe encore. C'est noté ici pour qu'on y pense si quelqu'un demande un document de 2023 qui n'aurait pas survécu à la migration.
