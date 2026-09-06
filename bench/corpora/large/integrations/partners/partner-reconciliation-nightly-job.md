---
name: partner-reconciliation-nightly-job
description: Le job nocturne app:partners:reconcile : comparaison complète des annonces avec correction journalisée, commissions dues et à recevoir, relevé mensuel
type: project
status: active
verified: 2026-06-27
---

# Réconciliation nocturne des partenaires

`app:partners:reconcile --partner <p> --fix` tourne à 03:00 pour chaque partenaire (Cargolink puis Fretzone, l'un après l'autre pour ne pas doubler la charge sur notre base). C'est le filet de sécurité de la synchro événementielle ([[partner-cargolink-sync-push-feed]]) et la source des chiffres de commission.

## Partie 1 : les annonces

1. Liste de nos chargements qui devraient avoir une annonce (`OPEN`/`BIDDING`, org avec le partenaire activé, visibilité compatible, type de véhicule supporté).

2. Liste des annonces actives sur notre compte chez le partenaire (paginée : Cargolink 200 par page, Fretzone 100).

3. Pour chaque chargement, comparaison du payload normalisé ([[partner-load-field-mapping-rules]]) avec l'annonce. Quatre cas :

- Chargement sans annonce : `create` (ou `relist` si un `external_id` existe).

- Annonce sans chargement actif : `delete`, sauf si le chargement a été attribué à un transporteur du partenaire (alors `awarded`).

- Champs différents : `update`, sauf si la ligne porte `stale_loop_suspected` ([[partner-incident-2026-04-fretzone-price-drift]]), auquel cas rien et une ligne dans le rapport.

- Identiques : rien.

4. Chaque correction est écrite dans `partner_reconcile_log` (date, `load_id`, partenaire, action, avant, après). Le rapport du matin dans `#integrations` compte les corrections par type.

Sans `--fix`, le job rapporte sans corriger ; c'est le mode utilisé en staging et pour les tests après un changement de mapping.

### Ordres de grandeur

| | Cargolink | Fretzone |
|---|---|---|
| Chargements à comparer | 4 000 à 6 000 | 2 500 à 3 500 |
| Appels partenaire | 25 à 35 | 30 à 40 |
| Durée | 3 à 5 min | 6 à 9 min (leur API est lente) |
| Corrections par nuit depuis mars 2026 | 0 à 6 | 2 à 15 |

Fretzone corrige plus parce que leur poll de 10 minutes laisse des `PENDING_PUSH` en fin de journée que personne n'a acquittés avant minuit, et parce qu'ils normalisent davantage.

## Partie 2 : les commissions

Pour chaque chargement passé `DISPATCHED` la veille avec un transporteur fantôme ou lié à un partenaire ([[partner-bid-relay-and-shadow-carriers]]) :

- Commission **due** au partenaire : montant de l'offre acceptée, pourcentage contractuel, zéro si la clause de relation existante s'applique (Cargolink : transporteur actif chez nous dans les 90 jours précédents).

- Commission **à recevoir** du partenaire (Fretzone uniquement) : annonces Fretzone attribuées à un transporteur Halden.

Écrit dans `partner_commissions` (une ligne par chargement, avec la règle appliquée). Un chargement annulé après dispatch (`CANCELLED` depuis `DISPATCHED`) génère une ligne négative le lendemain : les deux contrats ne facturent que l'exécuté, et on préfère écrire l'annulation que supprimer la ligne.

Le 2 de chaque mois, `app:partners:statement --month` produit le relevé par partenaire (PDF et CSV) envoyé à la finance et au partenaire. Le relevé du partenaire arrive de son côté ; la finance compare, et un écart de plus de 2 % ouvre un ticket chez le partenaire. Écarts constatés : 0,4 % en moyenne sur 2026, presque toujours des fuseaux horaires de fin de mois (un dispatch à 23:50 heure de Paris est le mois suivant pour Cargolink qui compte en UTC). On a aligné sur UTC dans le relevé en avril, l'écart est passé sous 0,1 %.

## Alertes

- `PartnerReconcileFailed` : le job ne s'est pas terminé (page l'astreinte si deux nuits de suite).

- `PartnerReconcileDrift` : plus de 50 corrections d'annonces en une nuit pour un partenaire. C'est le signe que la synchro événementielle a perdu des messages dans la journée ; notification, pas de page.

- `PartnerCommissionAnomaly` : montant journalier de commission au-dessus de trois fois la médiane sur 30 jours. Jamais déclenchée en vrai, une fois en test.

## Ce que le job ne fait pas

Il ne touche pas aux chargements rapatriés (`direction = 'inbound'`) : ceux-là suivent le pull, et un écart y signifie que le pull a raté un cycle, ce qui se voit ailleurs. Il ne recalcule pas les commissions des mois clos ; une correction rétroactive passe par une ligne d'ajustement datée du jour, jamais par une modification.

## Lire le rapport du matin

Le message dans `#integrations` a toujours la même forme :

```
[reconcile cargolink] 2026-06-26 03:04 → 03:08
compared 5 210 loads / 5 203 listings
create 2  relist 0  update 3  delete 1  awarded 0  skipped(loop) 0
commissions: due 41 loads 3 180.00 EUR (exempt 12) · receivable n/a
[reconcile fretzone] 03:08 → 03:16
compared 3 040 loads / 3 011 listings
create 6  relist n/a  update 7  delete 2  awarded 3  skipped(loop) 1
commissions: due 18 loads 1 440.00 EUR (exempt 0) · receivable 9 loads 610.00 EUR
```

Ce qu'on regarde : la durée (une dérive au-dessus de 10 minutes pour Fretzone annonce une lenteur de leur API), le total des corrections (au-dessus de 20, chercher pourquoi la synchro du jour a perdu des messages), `skipped(loop)` (chaque ligne est un chargement à regarder à la main, il y en a rarement plus d'une), et l'écart entre chargements et annonces comparés (une différence de plus de 1 % veut dire que des annonces existent chez le partenaire sans chargement actif chez nous, ou l'inverse, et le détail est dans `partner_reconcile_log`).

## Rejouer une nuit

`app:partners:reconcile --partner fretzone --date 2026-06-25` recalcule les commissions d'une date passée en mode rapport seul, sans toucher aux lignes existantes, et affiche l'écart avec ce qui a été écrit cette nuit-là. C'est ce que la finance demande quand le relevé du partenaire diffère du nôtre : on rejoue la date, on compare, et dans tous les cas vus l'écart venait d'un dispatch annulé le lendemain que l'un des deux comptait et pas l'autre.
