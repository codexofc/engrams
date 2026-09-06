---
name: data-retention-matrix
description: Platform retention per category: positions 90 days raw then trip summaries, PODs 10 years, chat 24 months, contact persons 13 months; enforced nightly
type: reference
status: active
verified: 2026-04-09
---

# Matrice de rétention (plateforme)

Cette matrice couvre les données de production de la plateforme : Postgres, stockage objet, index de recherche. L'entrepôt de données a ses propres règles (voir les notes du projet warehouse), qui s'appliquent après pseudonymisation. La version précédente est archivée dans [[retention-matrix-2025]].

Chaque ligne a un fondement : soit une obligation légale (durée fixée par la loi), soit une finalité contractuelle (on garde tant que c'est utile au service, et on écrit combien de temps c'est utile), soit un intérêt légitime documenté dans le registre ([[records-of-processing-register]]).

### La matrice

| Donnée | Table ou store | Durée | Fondement | Après la durée |
|---|---|---|---|---|
| Compte utilisateur actif | `users`, `organization_memberships` | tant que le compte existe | contrat | effacement sur demande ou 24 mois d'inactivité |
| Compte inactif | `users` | 24 mois sans connexion | contrat | pseudonymisation (nom, e-mail, téléphone remplacés) |
| Chargements, offres, missions | `loads`, `bids`, `assignments` | 10 ans | obligation comptable | conservés, les personnes de contact sont effacées à 13 mois |
| Personnes de contact sur un chargement | `loads.delivery_contact_*`, `pickup_contact_*` | 13 mois après livraison | intérêt légitime | colonnes mises à null |
| Factures, avoirs, paiements | `invoices`, `credit_notes`, `payments` | 10 ans | obligation comptable | conservés |
| Preuves de livraison (photos, signatures) | `hf-documents/pod/` | 10 ans | obligation contractuelle et litiges | suppression |
| Documents transporteur (licence, assurance) | `hf-documents/carrier/` | 5 ans après fin de relation | intérêt légitime (fraude) | suppression |
| Verdicts Verifid | `kyc_checks` | 5 ans après fin de relation | obligation (LCB-FT) | suppression |
| Positions GPS brutes | `positions` (partitionnée) | 90 jours | contrat (suivi en temps réel) | agrégées en `trip_summaries` (distance, durée, arrêts), sans trace fine |
| Résumés de trajets | `trip_summaries` | 24 mois | contrat (litiges, ETA) | suppression |
| Messages dispatcher-chauffeur | `messages` | 24 mois | contrat | suppression |
| Notes de support | outil de support | 36 mois | intérêt légitime | suppression par le fournisseur |
| Journal d'audit sécurité | `audit_events` | 13 mois en ligne, 6 ans en archive | intérêt légitime, obligation | voir la note IAM sur l'archive |
| Événements analytiques | ClickHouse | 13 mois | consentement | suppression |
| Exports demandés par un client | `hf-exports` | 7 jours | contrat | suppression automatique (règle de cycle de vie du bucket) |
| Messages EDIFACT archivés | `hf-edi-archive` | 10 ans | obligation contractuelle avec les chargeurs | suppression |
| Logs applicatifs | Loki | 30 jours | intérêt légitime | suppression |
| Sauvegardes Postgres | stockage de sauvegarde | 35 jours | continuité | suppression, donc une donnée effacée disparaît des sauvegardes sous 35 jours |

## Les décisions qui ont fait débat

**Positions à 90 jours.** Les dispatchers voulaient un an « pour les litiges ». Le DPO a fait remarquer qu'une trace fine des déplacements d'un chauffeur nommé pendant un an est une donnée de surveillance au sens plein, voir [[driver-position-legal-basis]]. Le compromis : 90 jours de brut, puis des résumés par trajet (départ, arrivée, distance, arrêts de plus de 15 minutes sans coordonnées précises) pendant 24 mois. Les litiges réels regardés sur 2025 se sont tous ouverts dans les 30 jours.

**Contacts de livraison à 13 mois.** Ce sont des personnes qui ne sont pas nos clients (le réceptionnaire chez le client du chargeur). On les garde le temps d'un cycle de contestation et d'une clôture annuelle, puis on les efface du chargement, qui reste. Le chargeur qui a besoin du nom au-delà l'a dans son propre système.

**EDIFACT à 10 ans.** Les messages contiennent des personnes de contact en clair et on ne peut pas les modifier (signature de l'échange). On a accepté la durée parce que les contrats avec les gros chargeurs l'exigent, et on a restreint l'accès au bucket à deux personnes et au processus de réconciliation.

## Application

`compliance:purge` tourne chaque nuit à 04:00, une classe `RetentionRule` par ligne de la matrice, chaque règle avec `--dry-run` par défaut qui écrit dans `retention_reports` le nombre de lignes concernées. La purge effective est activée règle par règle par `RETENTION_ENFORCE=positions,contacts,...`. En avril 2026, toutes les règles sont en mode effectif sauf `carrier_documents`, encore en dry-run parce que la « fin de relation » n'est pas encore un événement fiable dans les données (un transporteur peut rester six mois sans chargement puis revenir).

Le rapport du dry-run est relu chaque lundi par la personne d'astreinte conformité. Un écart de plus de 3 fois la médiane sur une règle bloque la purge de cette règle, en attendant qu'on comprenne.

## Lire le rapport du lundi

Le rapport `retention_reports` du lundi ressemble à ceci pour une règle :

```
rule: contacts_13m        mode: enforce
rows_matching: 41 208     median_4w: 39 900     ratio: 1.03     action: applied
rule: carrier_documents_5y   mode: dry_run
objects_matching: 2 140   median_4w: 2 100      ratio: 1.02     action: none (dry-run)
```

Un `ratio` au-dessus de 3 bloque la règle et pose une alerte. C'est arrivé une fois, en mars 2026, sur `contacts_13m` : 180 000 lignes au lieu de 40 000, parce qu'un import EDI de rattrapage avait créé 140 000 chargements anciens en une nuit. La purge a attendu qu'on confirme, puis a repris. Sans le seuil, elle aurait été correcte ; avec, on a su qu'il s'était passé quelque chose.
