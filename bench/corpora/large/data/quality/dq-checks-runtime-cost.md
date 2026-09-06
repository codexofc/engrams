---
name: dq-checks-runtime-cost
description: Mai 2026, les 370 règles coûtaient 48 minutes de CPU entrepôt par jour, 6 % du total, trois règles en faisaient 60 %, élagage à 310 règles et réécriture de cinq requêtes, 22 minutes par jour depuis, la revue de coût est obligatoire au-dessus de 10 s par exécution, HF-4550
type: project
status: active
verified: 2026-06-26
---

# Ce que coûtent les règles, et ce qu'on a élagué

## Le constat (avril 2026)

Le panneau « temps d'exécution par règle » du tableau de bord ([[dq-framework-overview]]) montrait 48 minutes de temps d'entrepôt par jour pour 370 règles, soit 6 % du CPU de l'entrepôt, sur une facture mensuelle de 14 000 EUR environ : 850 EUR par mois de qualité des données, ce qui est défendable. Ce qui l'était moins : trois règles faisaient 60 % du total.

### Les dix règles les plus coûteuses, avril 2026

| Règle | Type | Table | Exécutions / jour | Temps total / jour | Cause |
|---|---|---|---|---|---|
| `lsh_vs_domain_events` | réconciliation | `core.load_status_history` (210 M) | 24 | 14 min | jointure complète des deux côtés à chaque heure |
| `positions_unique` | `unique` | `raw.driver_positions` (2,1 Md) | 96 | 9 min | `count(distinct)` sur 90 jours, à chaque lot |
| `bids_load_exists` | requête | `core.bids` (41 M) | 96 | 6 min | anti-jointure sur toute la table à chaque run |
| `lsh_pk_unique` | `unique` | `core.load_status_history` | 96 | 3 min | toute la table |
| `loads_count_vs_cdc` | réconciliation | `core.loads` / `raw` | 96 | 2 min | |
| `bids_pk_unique` | `unique` | `core.bids` | 96 | 2 min | |
| `invoices_status_transitions` | requête | `core.invoice_status_history` | 24 | 1 min 30 | fonction fenêtre sur tout l'historique |
| `carriers_one_current_row` | requête | `core.carriers` | 24 | 40 s | |
| 60 règles générées `schema` + `freshness` | | | 24 à 288 | 3 min cumulées | pas cher unitairement, nombreuses |
| les 300 autres | | | | 6 min cumulées | |

`positions_unique` sur `raw` contredisait le principe « pas de règle sur `raw` » et avait été ajoutée « pour voir » en novembre. Elle n'avait jamais rien trouvé.

## Ce qu'on a fait (HF-4550, mai 2026)

1. **Fenêtrer les règles de ligne.** `unique`, `not_null` et les requêtes de ligne sur les tables de plus de 10 M de lignes ne regardent que les partitions touchées par le dernier run du modèle (marmot expose `changed_partitions` au post-hook) plus la partition courante. Une règle qui doit voir toute la table (les réconciliations, `carriers_one_current_row`) le déclare avec `scope: full` et passe en quotidien à 06:15 au lieu de post-hook. `lsh_pk_unique` est passée de 2 s à 60 ms par exécution.

2. **Réécrire les cinq pires requêtes.** `lsh_vs_domain_events` compare des comptes par heure et par statut, pas des lignes, et seulement pour les 3 dernières heures : 35 s à 1,2 s. `bids_load_exists` ne vérifie que les enchères insérées depuis le dernier run : 4 s à 80 ms. `invoices_status_transitions` fenêtrée sur 7 jours de transitions.

3. **Supprimer 60 règles.** Critères ([[quality-owner-preferences]]) : jamais en échec depuis octobre, pas d'`origin`, et l'invariant déjà couvert par une autre règle ou hors principe. `positions_unique` en tête. Onze règles `accepted_values` sur des colonnes techniques (`inserted_by`, `batch_id`) qui vérifiaient que l'ingestion écrivait ce que l'ingestion écrit. Quarante règles `schema` sur des tables `ops` sans consommateur. Aucune règle avec `origin` n'a été supprimée, quelle que soit sa fréquence d'échec (zéro pour la plupart) : elles existent pour une raison qu'on a payée.

4. **Passer 20 règles en `info`** plutôt que de les supprimer, quand quelqu'un voulait garder la mesure sans l'alerte.

## Résultat

| | Avril | Juin |
|---|---|---|
| règles | 370 | 310 |
| temps d'entrepôt par jour | 48 min | 22 min |
| part du CPU entrepôt | 6 % | 2,7 % |
| règle la plus coûteuse | 14 min / jour | 2 min / jour (`lsh_vs_domain_events`, encore) |
| alertes `warn` par mois | 52 | 38 |
| incidents attrapés | 1 | 1 |

Les 30 % d'alertes en moins sont surtout les règles passées en `info`. Rien de ce qui a été supprimé n'a manqué ; c'est la ligne attrapé / manqué des fiches ([[dq-scorecards-per-domain]]) qui le dit, et elle est à 0 manqué depuis mai.

## La règle de coût, depuis

Toute règle dont une exécution dépasse 10 s, ou dont le total dépasse 1 minute par jour, passe par une revue de coût avant d'entrer dans le chemin post-hook : soit elle est fenêtrée, soit elle est quotidienne en `scope: full`, soit elle justifie son coût dans la description. `dq-runner lint` calcule une estimation à partir de la taille des tables et du type de règle et refuse au-dessus du seuil sans le champ `cost_reviewed: <date>`.

Le panneau a un `warn` à 30 minutes de temps d'entrepôt par jour. On n'y est pas retourné. La prochaine table qui fera exploser le compteur est probablement `raw.driver_positions`, si quelqu'un veut à nouveau une règle dessus ; la réponse sera la même qu'en mai.
