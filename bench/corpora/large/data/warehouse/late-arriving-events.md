---
name: late-arriving-events
description: Late events (mobile buffering up to 7 days, replays) handled by recomputing touched partitions up to 35 days back, weekly beyond
type: project
status: active
verified: 2026-03-24
---

## Le problème

Un événement peut arriver longtemps après son `occurred_at` :

- l'app conducteur met en mémoire les événements hors réseau et les envoie à la reconnexion ; 3 % des événements conducteur arrivent plus d'une heure après, 0,4 % plus d'un jour après, et le maximum observé est 7 jours (un conducteur en zone blanche en Roumanie, puis en congé) ;
- les rejeux CDC après incident ([[duplicate-bids-incident-2026-01]]) ;
- les corrections manuelles dans l'app (un POD accepté avec effet rétroactif).

Un modèle incrémental qui ne recalcule que « depuis le dernier run » rate ces lignes si la partition concernée est plus ancienne. C'est comme ça que `rank_at_close` manque sur 2 % des enchères de [[bids-fact-model]] : l'événement `bid.ranked` est arrivé après le run et la partition n'a pas été retouchée.

## Ce que marmot fait

Pour un modèle `incremental`, à chaque run ([[marmot-model-runner]]) :

1. lister les partitions des sources ayant reçu des lignes depuis le dernier run (`SELECT DISTINCT partition FROM raw.x WHERE received_at > last_run`), et non les lignes par `occurred_at` ;
2. recalculer ces partitions entières, quelle que soit leur ancienneté, jusqu'à 35 jours en arrière ;
3. au-delà de 35 jours, noter la partition dans `marmot._deferred` et la traiter au passage hebdomadaire du dimanche 04:00, qui recalcule toutes les partitions différées et, une fois par mois, toutes les partitions des 400 jours.

Le coût : un run de 10 minutes recalcule en médiane 2 partitions mensuelles par modèle (le mois courant et le précédent), parfois 3 ou 4 après un rejeu. Le recalcul d'une partition de `core.loads` prend 25 s, celui de `core.bids` 40 s.

## Pourquoi 35 jours

Mesuré sur `received_at - occurred_at` en février 2026 : 99,97 % des événements arrivent sous 35 jours. Les 0,03 % restants sont des rejeux ou des corrections, traités le dimanche. Un seuil de 7 jours laissait 0,3 % aux passages hebdomadaires, et les analystes voyaient des chiffres bouger le lundi.

## Colonnes qui restent fausses entre deux passages

`time_to_first_bid_s` dans `core.loads` peut être surestimé pendant quelques heures si l'événement de la première enchère est en retard ; `bid_count` sous-estimé de même. Les marts quotidiens portent une colonne `computed_at` et le tableau de bord affiche « données au <date> » à partir d'elle. On a renoncé à une notion de « partition définitive » : rien n'est définitif, tout est recalculable, et c'est [[backfill-runbook]] qui décrit comment forcer un recalcul.

## Ce qu'on a refusé

Un flux séparé « événements en retard » traité par des `INSERT` correctifs ligne à ligne. Simple sur le papier, mais chaque modèle aurait eu deux chemins de calcul, et les deux auraient divergé. Recalculer la partition entière est plus lent et strictement plus simple.
