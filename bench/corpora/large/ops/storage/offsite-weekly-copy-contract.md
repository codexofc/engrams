---
name: offsite-weekly-copy-contract
description: La copie hors site hebdomadaire est un service du prestataire du datacentre, disques chiffrés emportés chaque mardi vers son second site à 90 km, 52 semaines conservées, cinq buckets dans le périmètre, restauration à la demande sous 48 h contractuelles (4 h mesurées pour un petit bucket), et ce que le contrat ne couvre pas
type: reference
status: active
verified: 2026-05-27
---

## Le service

Le prestataire du datacentre propose une copie hors site sur disques : chaque mardi matin, un technicien branche une baie de disques amovibles sur le réseau de gestion, un job de notre côté (`offsite-sync`, sur `ops-tools`) écrit un incrémental chiffré des buckets du périmètre, le technicien emporte la baie vers le second site du prestataire à 90 km, et la baie de la semaine précédente revient. Deux baies en rotation, 52 semaines conservées sur des disques stockés au second site (une baie « année » qui reçoit un consolidé mensuel, plus les hebdomadaires du trimestre courant). Contrat annuel, reconduit en janvier 2026, dans la ligne « stockage » des coûts.

C'est la troisième copie ([[backup-inventory-and-retention]]), celle qui survit à la perte du datacentre. Les deux premières sont dans les racks.

## Le périmètre

| Bucket | Volume total | Incrémental hebdomadaire typique |
|---|---|---|
| `hf-documents-prod` | 9,1 TB | 60 GB |
| `hf-pg-backups-prod` | 4,8 TB (35 jours glissants) | 4,8 TB (tout change, c'est du WAL) |
| `hf-vault-snapshots` | 2 GB | 2 GB |
| `hf-ml-artifacts-prod` | 210 GB | 5 GB |
| `hf-warehouse-cold-prod` | 31 TB | 80 GB, plus les réécritures (6 TB en mars 2026) |

Environ 5 TB par semaine en régime normal, 6 heures d'écriture sur la baie. La semaine de la réécriture des parties froides ([[incident-2026-03-appliance-replication-lag]]) a demandé 11 TB et deux baies, ce qui était prévu au contrat (« jusqu'à 20 TB par passage sur demande 48 h avant »).

Hors périmètre : Velero, etcd, Tempo, tuiles, exports, `hf-ops-misc`. Tout ce qui se reconstruit depuis Git ou depuis les autres copies.

## Le chiffrement

Tout ce qui est écrit sur la baie est chiffré par `offsite-sync` avant d'atteindre les disques, avec les clés décrites dans [[backup-encryption-and-key-custody]]. Le prestataire transporte des disques qu'il ne peut pas lire. La baie elle-même est aussi chiffrée par son propre contrôleur, avec une clé du prestataire : ceinture et bretelles, la nôtre est celle qui compte.

## La restauration

Contrat : la baie demandée est ramenée sur site sous 48 h ouvrées. Mesuré une fois en vrai, en 2025, pour un test sur `hf-vault-snapshots` : demande le lundi 10:00, baie branchée le mardi 09:30 (le technicien passait de toute façon), déchiffrement et lecture 30 minutes, 4 heures au total pour 2 GB dont 23 h 30 d'attente. Pour 31 TB de parties froides, la lecture depuis la baie est la contrainte : environ 250 MB/s, soit 35 heures, plus le transport. C'est le « trois jours » du plan de l'exercice de novembre 2026 ([[restore-drill-2026-05]] a laissé ce point pour plus tard).

## Ce qu'on vérifie

- Chaque mardi, `offsite-sync` écrit un manifeste (liste des objets, tailles, hachés) sur la baie et une copie dans `hf-ops-misc/offsite-manifests/`. Le job vérifie que le nombre d'objets écrits correspond au manifeste et lit 200 objets au hasard depuis la baie pour comparer les hachés. Un écart est un `page`.

- Le prestataire signe un bordereau de départ et un bordereau d'arrivée au second site, avec le numéro de série de la baie. Les bordereaux sont dans le dossier du contrat. Une baie partie sans bordereau d'arrivée sous 24 h est un ticket, arrivé une fois (oubli administratif, la baie était là).

- Une fois par an, une baie « année » est rapatriée et relue intégralement (janvier 2026 : 38 TB relus en 44 heures, 0 erreur).

## Ce que le contrat ne couvre pas

- La disponibilité du second site en cas de sinistre régional. À 90 km, une inondation du Rhône touche les deux ; un incendie non. C'est accepté et écrit dans l'analyse de risque.

- Une restauration plus rapide que 48 h. Il n'y a pas d'option « urgence ». Si c'est le datacentre qui a brûlé, 48 h ne seront pas le problème le plus long.

- La confidentialité au-delà du transport : le prestataire s'engage sur la garde des disques, pas sur leur contenu, qu'il ne peut pas lire. Si nos clés sont perdues, les disques sont du métal.

## Ce qu'on a écarté

Une copie hors site vers un stockage objet distant chez un fournisseur cloud. Étudié en 2025 : 31 TB de froid à sortir puis à garder coûtent plus que le contrat sur disques, et la sortie en cas de restauration (l'egress) coûterait plus que la baie. Le compromis est un délai de 48 h contre un coût connu, et une baie de disques ne dépend d'aucun compte, d'aucune clé d'API et d'aucune connexion internet le jour où on en a besoin. Revu chaque année à la reconduction, maintenu en 2026 pour la même raison.
