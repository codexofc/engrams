---
name: incident-2025-11-migration-lock-loads
description: Nov 2025 deploy where a migration backfilling loads.pickup_window_start held an ACCESS EXCLUSIVE lock for 14 minutes and blocked every API request, led to the no-data-in-migrations rule and lock_timeout
type: project
status: active
verified: 2025-11-28
---

# Incident 2025-11-06: migration lock on `loads`

## Résumé

Déploiement de 14 h 02, migration `Version20251105163012` incluse dans HF-1160 (remplacement de `pickup_date` par `pickup_window_start` et `pickup_window_end`). L'API a été indisponible de 14 h 03 à 14 h 17. Pas de perte de données. 14 minutes de 503 pour tous les clients, 1 200 chauffeurs environ en synchronisation ratée, web indisponible.

## Chronologie (heure de Paris)

- 14 h 02 : ArgoCD lance le Job de migration (hook `PreSync`).

- 14 h 03 : la migration exécute `ALTER TABLE loads ADD COLUMN pickup_window_start timestamptz` (rapide, 30 ms) puis dans la même transaction `UPDATE loads SET pickup_window_start = pickup_date::timestamptz` sur 3,9 M de lignes.

- 14 h 03 : l'`ALTER TABLE` a pris un verrou `ACCESS EXCLUSIVE` sur `loads`. Il n'est relâché qu'au commit. L'`UPDATE` met 14 minutes. Pendant ce temps, tout `SELECT` sur `loads` attend le verrou.

- 14 h 04 : `ApiErrorRateHigh` et `ApiLatencyP99High` déclenchent. php-fpm sature (`pm.max_children` atteint sur tous les pods), le readiness probe `/health` (qui fait un `SELECT 1`, pas un select sur `loads`) répond encore, donc Kubernetes ne fait rien, ce qui est en fait correct.

- 14 h 06 : on-call identifie le Job de migration. Hésitation : tuer le Job laisserait la transaction en cours côté PostgreSQL jusqu'à ce que le serveur voie la déconnexion. `pg_terminate_backend` aurait annulé 3 minutes d'`UPDATE` et il en restait 11 à venir, on ne le savait pas.

- 14 h 08 : décision de laisser finir plutôt que de tuer, parce que `pg_stat_progress` ne donne rien pour un `UPDATE` et qu'on n'avait aucune idée du temps restant. Mauvaise décision rétrospectivement, l'annulation aurait pris 2 minutes et on aurait perdu 6 minutes au lieu de 11.

- 14 h 17 : commit. Les requêtes en attente s'exécutent d'un coup, pic de charge de 40 s, puis retour à la normale.

- 14 h 25 : incident clos côté utilisateurs. Post-mortem le 10 novembre.

## Pourquoi ça a passé la revue

La migration avait été testée sur staging, où `loads` a 80 000 lignes : 4 secondes. Personne n'a fait le produit en croix. La revue a regardé le SQL, l'a trouvé correct (il l'était), et n'a pas pensé au verrou.

## Ce qui a changé

1. **Plus jamais de données dans une migration de schéma.** Les backfills sont des commandes `app:backfill:*` par lots avec `--batch-size` (5 000 par défaut) et une pause, lancées après le déploiement, pendant que les deux colonnes coexistent. Règle formalisée dans [[doctrine-migration-workflow]].

2. **`SET lock_timeout = '5s'`** en tête de chaque migration. Si l'`ALTER` ne peut pas prendre son verrou en 5 s (à cause d'une transaction longue en cours), la migration échoue et le déploiement s'arrête proprement au lieu de mettre tout le monde en file d'attente derrière le verrou.

3. **`activeDeadlineSeconds: 600`** sur le Job de migration. Une migration qui dure plus de 10 minutes est une erreur, pas une migration.

4. **Le readiness probe reste sur `SELECT 1`.** On a discuté de le faire toucher `loads`. Non : si la base est bloquée, retirer les pods du service ne libère rien et fait perdre les réponses d'erreur propres.

5. **Estimation obligatoire dans la PR** pour toute migration touchant `loads`, `bids`, `load_events`, `invoices` : nombre de lignes en prod (chiffre du dashboard PostgreSQL) et durée estimée. Une ligne dans le template de PR.

## Ce qu'on a appris sur PostgreSQL au passage

- `ALTER TABLE ... ADD COLUMN` avec un `DEFAULT` constant est instantané depuis PostgreSQL 11 (le défaut est stocké dans le catalogue), mais un `DEFAULT now()` ou tout défaut volatile réécrit la table.

- `ADD COLUMN` sans défaut suivi d'un `UPDATE` dans la même transaction cumule le pire des deux : le verrou exclusif de l'`ALTER` plus la durée de l'`UPDATE`.

- L'ordre des verrous compte : une transaction qui attend `ACCESS EXCLUSIVE` bloque aussi toutes celles qui arrivent derrière, même les `SELECT`, parce que PostgreSQL fait la queue dans l'ordre. C'est ce qui a transformé une migration lente en indisponibilité totale.

## Lien avec la suite

Cet incident est la raison de la règle "estimation dans la PR" et de la commande de backfill. Il est aussi la raison pour laquelle [[audit-log-table-partitioning]] a été fait en trois déploiements au lieu d'un.
