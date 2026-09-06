---
name: storage-capacity-plan-2026
description: Plan de capacité arrêté en avril 2026 (HF-4630): 46 TB sur 60 et +90 GB/semaine, le froid fait deux tiers, un tiroir de 12 disques par appliance en septembre
type: project
status: active
verified: 2026-05-27
---

# Capacité du stockage objet, 2026

## Où on en est (avril 2026)

| | `stash-a` | `stash-b` |
|---|---|---|
| utilisable | 60 TB | 60 TB |
| utilisé | 46,4 TB (77 %) | 46,1 TB (répliqué, légère avance de suppression) |
| seuil `warn` | 75 % (dépassé depuis le 2026-04-02) | |
| seuil `page` | 85 % | |
| croissance | +90 GB par semaine en moyenne sur 6 mois, +150 les semaines de fin de mois | |

Détail par bucket dans [[object-store-buckets-and-layout]]. Deux buckets font l'essentiel : `hf-warehouse-cold-prod` (31 TB, +80 GB par semaine) et `hf-documents-prod` (9,1 TB, +60 GB par semaine, en léger ralentissement depuis que les pièces jointes ont disparu des notifications). Le reste est stable ou borné par sa rétention.

À 90 GB par semaine, 85 % (51 TB) est atteint vers la fin janvier 2027 et 100 % en octobre 2027. Ce n'est pas urgent ; c'est dans dix mois, et un tiroir de disques se commande avec six semaines de délai. Le plan a été arrêté en avril pour ne pas y penser en janvier.

## Les options regardées

### Comparaison

| Option | Effet | Coût | Ce qu'on perd |
|---|---|---|---|
| A. Réduire le froid de l'entrepôt de 400 à 270 jours | −9 TB une fois | 0 | 130 jours d'historique brut, demandé par personne mais utilisé par l'équipe ML pour deux backtests en 2025 |
| B. Compression ZSTD(3) du froid (fait en mars) | −38 % sur `payload`, environ −8 TB | 0, déjà fait | +15 % de temps de lecture sur le froid |
| C. Sortir le froid vers un stockage distant | −31 TB dans les racks | facturé au TB-mois, plus l'egress à chaque lecture | la latence de lecture, une dépendance externe, et le coût variable |
| D. Un tiroir d'extension de 12 disques de 8 TB par appliance | +30 TB par appliance (code d'effacement 16+8 → un groupe de plus) | environ 11 000 EUR par tiroir, deux tiroirs | rien |
| E. Remplacer les appliances par le modèle suivant | +120 TB | six chiffres | prématuré, cycle matériel en 2028 |

B est déjà dans les chiffres d'avril : les 31 TB sont post-compression (c'était 39). Sans B, le plan aurait été urgent.

## La décision (HF-4630)

D, deux tiroirs, commandés en mai, installés en septembre 2026 pendant une fenêtre de maintenance (le tiroir s'ajoute à chaud, mais la redistribution du code d'effacement sur le nouveau groupe prend 30 heures par appliance à débit réduit, et on préfère le faire un week-end). Après installation : 90 TB utilisables par appliance, 46 TB utilisés, 51 %, et le seuil de 75 % repoussé à 2029 au rythme actuel, c'est-à-dire après le cycle matériel de 2028 où la question se reposera avec des disques plus gros.

Pourquoi pas A : 130 jours de brut sont un actif pour deux backtests par an et un passif de 9 TB. 9 TB coûtent un tiers de tiroir, soit environ 3 700 EUR une fois. L'équipe ML a estimé le coût de refaire un backtest sans le brut (reconstruire depuis les agrégats) à une semaine de travail. L'arithmétique est claire. La rétention à 400 jours reste ([[retention-by-data-class]]), et sera rediscutée si la croissance du froid dépasse 120 GB par semaine, ce qui serait le signe d'un usage nouveau plutôt que de la croissance du métier.

Pourquoi pas C : le froid est lu plus qu'on ne le pense (les requêtes analystes sur plus de 90 jours, 8 % des requêtes, et les backfills), et chaque lecture depuis un stockage distant paierait de l'egress. La copie hors site ([[offsite-weekly-copy-contract]]) reste sur disques pour la même raison.

## Ce qui a été fait en attendant

- Les téléchargements partiels abandonnés sur `hf-documents-prod` (140 GB) supprimés par la règle d'abandon à 2 jours ([[bucket-versioning-and-lifecycle-rules]]).

- Les versions non courantes de `hf-ops-misc` (400 GB d'un tarball répété) expirées.

- `hf-tiles-prod` ramené de 60 à 30 jours : −90 GB, sans effet mesurable sur le taux de succès du cache (98,1 % avant, 97,9 % après).

- La seconde base PostgreSQL quotidienne à 13:00 ([[restore-2026-02-postgres-pitr-billing]]) ajoute 60 GB par jour dans `hf-pg-backups-prod`, mais la rétention de 35 jours borne l'effet à +2 TB en régime établi, comptés dans le plan.

Total des gains : environ 0,7 TB, symbolique par rapport aux 31 TB de froid, mais c'est le ménage qu'on fait avant d'acheter des disques, pour ne pas acheter des disques pour du désordre.

## La courbe qui a décidé

### Utilisation de `stash-a`, fin de mois

| Mois | Utilisé | Dont froid entrepôt | Dont documents |
|---|---|---|---|
| 2025-10 | 41,2 TB | 36,0 (avant compression) | 8,1 |
| 2025-12 | 43,0 | 37,2 | 8,5 |
| 2026-02 | 44,8 | 38,4 | 8,8 |
| 2026-03 | 38,9 (après ZSTD) | 30,2 | 8,9 |
| 2026-04 | 46,4 (réécriture en cours, anciennes parties pas encore supprimées) | 31,0 | 9,1 |
| 2026-05 | 46,9 | 31,3 | 9,2 |

La ligne de mars montre l'effet de la compression, celle d'avril la suppression différée des anciennes parties par le TTL ClickHouse (les 9 TB de suppressions de la réécriture ont été comptés une semaine de plus que prévu, ce qui a déclenché le `warn` à 75 % qu'on aurait sinon évité jusqu'en juin). Le `warn` reste actif jusqu'à septembre en connaissance de cause, avec un silence documenté renouvelé chaque mois, ce qui est la seule alerte de stockage silencée et la raison pour laquelle le silence a une date de fin.

## Ce qui n'est pas dans ce plan

Les disques NVMe des nœuds (entrepôt chaud, PostgreSQL, brokers torrent) : ce sont des plans par équipe, et le broker a le sien depuis mai. Les sauvegardes : elles sont dans les buckets et donc dans les chiffres ci-dessus. Le second site du prestataire : ses baies sont dimensionnées par le contrat, 20 TB par passage, revu à la reconduction.
