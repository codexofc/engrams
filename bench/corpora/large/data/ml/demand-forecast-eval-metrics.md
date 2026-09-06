---
name: demand-forecast-eval-metrics
description: How the demand forecast is scored: WAPE per horizon weighted by volume, bias, naive same-weekday baseline, why not MAPE
type: reference
status: active
verified: 2026-03-13
---

Définitions utilisées par le [[backtesting-framework]] pour [[demand-forecast-model]]. Elles sont dans `mlkit.metrics.demand` et c'est le code qui fait foi ; cette note explique les choix.

## WAPE

`WAPE = sum(|y - ŷ|) / sum(y)` sur toutes les paires (cluster, jour) de la fenêtre, par horizon. Pondérée par construction : un cluster à 300 chargements par jour pèse 300 fois plus qu'un cluster à 1. C'est ce qu'on veut : l'usage commercial et pricing porte sur le volume.

Pas de MAPE : sur les clusters à 0 ou 1 chargement par jour (40 % des clusters, 3 % du volume), la MAPE est indéfinie ou explose, et une moyenne de MAPE par cluster récompense un modèle qui prédit 0 partout sur les petits clusters.

## Biais

`biais = sum(ŷ - y) / sum(y)`, signé. Un biais positif signifie une surestimation. Suivi par horizon et par semaine ; le biais des semaines fériées est reporté séparément parce qu'il domine le total.

## Référence naïve

Prévision = valeur observée le même jour de semaine, une semaine avant (`y[t-7]`). À J+7 c'est le mieux qu'un tableur fasse, et le modèle doit la battre sur chaque horizon pour être promu (règle de tolérance de [[model-registry-conventions]] : WAPE du candidat à +1 point du modèle courant maximum, et strictement sous la référence naïve).

Une deuxième référence, moyenne des 4 mêmes jours de semaine précédents, est reportée pour information ; elle est meilleure que la première sur les petits clusters et moins bonne sur les gros.

## Segments reportés

- Les 200 plus gros clusters (58 % du volume) : WAPE à part, parce que c'est ce que le commercial regarde.
- Par pays d'origine.
- Par bande de distance.
- Semaines fériées contre semaines normales.
- Clusters de moins de 6 mois (prévus par la moyenne de leur groupe, pas par le modèle) : reportés mais hors du score de promotion.

## Fenêtre

Backtest glissant : pour chaque semaine des 12 dernières, entraînement sur les 2 ans précédant le lundi, prévision des 14 jours suivants, comparaison aux observés. 12 fenêtres, métriques moyennées, et l'écart-type entre fenêtres est reporté (une WAPE de 18 % ± 6 selon la semaine en dit plus que 18 % seul).

## Ce que la métrique ne mesure pas

L'utilité pour le commercial, qui est : les relances envoyées sur une ligne prévue en tension ont-elles amené des enchères ? Cette mesure existe côté commercial (taux de réponse aux relances ciblées, 14 % contre 6 % pour les relances non ciblées, T1 2026) et n'est pas dans le backtest, parce qu'elle dépend de ce que le commercial fait de la prévision.
