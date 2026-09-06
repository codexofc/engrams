---
name: backup-inventory-and-retention
description: Inventaire de ce qui est sauvegardé, par quoi, où, à quelle fréquence, combien de temps, et la dernière restauration réelle; les trois choses non sauvegardées
type: reference
status: active
verified: 2026-06-30
---

# Inventaire des sauvegardes

La table ci-dessous est la référence. Elle est relue à chaque exercice de restauration ([[restore-drill-2026-05]] pour le dernier) et à chaque revue trimestrielle du stockage. Une ligne qui n'a pas été restaurée au moins une fois depuis douze mois est en gras dans la version du dépôt (`halden-infra/storage/BACKUPS.md`), pas ici.

## Ce qui est sauvegardé

### Table d'inventaire

| Quoi | Par quoi | Vers | Fréquence | Rétention | Dernière restauration réelle |
|---|---|---|---|---|---|
| PostgreSQL principal (API, auth, facturation) | opérateur, archivage WAL continu + base | `hf-pg-backups-prod` | WAL en continu, base quotidienne 01:00 | 35 jours (PITR) | [[restore-2026-02-postgres-pitr-billing]] |
| PostgreSQL entrepôt de métadonnées marmot | même mécanisme | même bucket, préfixe `marmot/` | idem | 35 jours | exercice mai 2026 |
| objets Kubernetes `hf-main` | Velero | `hf-velero-prod` | toutes les 6 h | 14 jours | exercice mai 2026 |
| volumes Longhorn (Redis, RabbitMQ, registre) | Velero + CSI | `hf-velero-prod` | quotidien 02:30 | 30 jours | exercice mai 2026 (RabbitMQ) |
| etcd RKE2 | natif RKE2 | `hf-etcd-snapshots` | toutes les 6 h | 30 jours | avril 2026, cluster jetable |
| vault (secrets) | instantané natif | `hf-vault-snapshots` | horaire | 90 jours | exercice mai 2026 |

### Table d'inventaire, suite : documents et entrepôt

| Quoi | Par quoi | Vers | Fréquence | Rétention | Dernière restauration réelle |
|---|---|---|---|---|---|
| documents (`hf-documents-prod`) | versionnage du bucket + réplication + copie hors site | `stash-b`, copie hors site | continu, hebdomadaire hors site | versions 90 jours, hors site 52 semaines | [[restore-2025-11-documents-prefix-deleted]] |
| entrepôt ClickHouse, données chaudes | instantanés incrémentaux natifs | `hf-warehouse-cold-prod`, préfixe `backups/` | quotidien 03:30 | 35 jours | décembre 2025, `raw.bids` |
| entrepôt ClickHouse, parties froides | déjà sur l'objet ; réplication + hors site | `stash-b`, hors site | continu, hebdomadaire | versions 30 jours | jamais restauré, voir ci-dessous |

### Table d'inventaire, fin : bus, Git, ML

| Quoi | Par quoi | Vers | Fréquence | Rétention | Dernière restauration réelle |
|---|---|---|---|---|---|
| topics torrent | non sauvegardés, réplication 3 | | | rétention des topics | sans objet |
| état du connecteur CDC et registre de schémas | export | `hf-torrent-snapshots` | horaire | 30 jours | exercice mai 2026 (registre) |
| dépôts Git | miroir sur `ops-tools` + hébergeur | `hf-ops-misc/git-mirror/` | horaire | 30 jours | 2025, pour un dépôt supprimé par erreur |
| artefacts ML | immuables par hash | `hf-ml-artifacts-prod`, hors site | à l'écriture | illimitée sauf élagage manuel | jamais (le registre pointe toujours dessus) |

## Ce qui n'est pas sauvegardé, et pourquoi

- **Les topics torrent.** Trois réplicas sur deux racks, 30 jours de rétention sur `domain.*`. Une sauvegarde d'un journal d'événements qui a déjà trois copies et dont le contenu est projeté dans PostgreSQL et l'entrepôt serait une quatrième copie de quelque chose qu'on peut reconstruire. Si les cinq brokers brûlent, l'entrepôt a `raw.*` avec 400 jours, et la reconstruction des projections se fait depuis là. Décision de 2024, revue en 2026, maintenue.

- **Les tuiles de carte** (`hf-tiles-prod`). Cache, reconstruit en 6 heures depuis la source cartographique.

- **`platform-staging`** et tout ce qui est reconstruit depuis Git et depuis une restauration de prod chaque nuit.

## Où sont les trois copies

Pour chaque ligne qui compte, la question posée à l'exercice est « où sont les trois copies et laquelle survit à quoi » :

1. la donnée vivante (PostgreSQL sur NVMe, les documents sur `stash-a`) ;

2. la sauvegarde ou la réplique dans l'autre rack (`stash-b` reçoit tout ce qui est sur `stash-a`, y compris les buckets de sauvegarde) ;

3. la copie hors site hebdomadaire ([[offsite-weekly-copy-contract]]), qui contient `hf-documents-prod`, `hf-pg-backups-prod`, `hf-vault-snapshots`, `hf-ml-artifacts-prod` et `hf-warehouse-cold-prod`, et pas le reste.

Le froid de l'entrepôt est le point discuté : 31 TB qui partent hors site chaque semaine (en incrémental, 80 GB de nouveau par semaine, donc peu de volume réel), jamais restaurés, parce qu'une restauration de 31 TB depuis le hors site prend trois jours et qu'on ne l'a pas encore fait « pour voir ». C'est la première ligne du plan de l'exercice de novembre 2026.

## Chiffrement

Tout ce qui quitte le rack est chiffré avant de partir, voir [[backup-encryption-and-key-custody]]. Ce qui reste dans les racks est chiffré au repos par les appliances, avec des clés qu'elles gèrent elles-mêmes.

## Revue

Trimestrielle, une heure, l'astreinte stockage et une personne de chaque équipe propriétaire : la table ci-dessus ligne par ligne, la colonne « dernière restauration réelle », les tailles ([[storage-capacity-plan-2026]]) et les rétentions ([[retention-by-data-class]]). Le compte rendu est le diff de `BACKUPS.md`.
