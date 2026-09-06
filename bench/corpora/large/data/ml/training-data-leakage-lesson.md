---
name: training-data-leakage-lesson
description: The 2025 demand model at 6 % WAPE offline and 31 % live because a feature looked into the future, and the three checks since
type: feedback
status: active
verified: 2025-12-09
---

## Ce qui s'est passé

Première version du modèle de prévision de demande ([[demand-forecast-model]]), septembre 2025. WAPE de 6 % à J+7 en validation, contre 27 % pour la référence naïve. Trop beau, et personne ne l'a dit à voix haute. Mis en production, WAPE réelle de 31 %, pire que la référence.

La feature coupable : `active_carriers_on_lane`, le nombre de transporteurs ayant enchéri sur le cluster « dans les 7 jours ». Calculée dans le jeu d'entraînement par rapport à la date **prévue**, pas à la date de **calcul** de la prévision. Pour une prévision à J+7, elle contenait les transporteurs qui allaient enchérir sur les chargements qu'on essayait de prévoir. En production, la valeur disponible est celle à la date de calcul, sans les sept jours futurs.

Une deuxième fuite, plus petite, dans le modèle d'ETA de l'époque : `carrier_lateness_90d` calculée avec une fenêtre qui incluait le chargement lui-même.

## Pourquoi ce n'était pas visible

- La validation était un découpage aléatoire des lignes, pas un découpage dans le temps. Une ligne de test avait ses voisines temporelles dans l'entraînement.
- La feature était construite par une jointure sur `date` sans `<`, écrite vite dans un notebook.
- Un score très supérieur à la référence a été pris pour une bonne nouvelle.

## Les trois contrôles

1. **Jointure point-in-time obligatoire.** `build_training_set` du paquet `mlkit` ([[feature-store-design]]) exige une colonne de temps par ligne et joint chaque feature avec `feature_date < row_time` (strict pour les features quotidiennes, `<=` pour les événements horodatés à la seconde). Un jeu d'entraînement construit autrement ne peut pas être enregistré.
2. **Validation temporelle.** Le [[backtesting-framework]] découpe par date : entraînement jusqu'à T, test de T à T+14 jours, en glissant. Un découpage aléatoire n'existe pas dans l'outil.
3. **Le test du « trop beau ».** Un modèle candidat dont la métrique bat la référence naïve de plus de 50 % en relatif est bloqué à l'enregistrement avec le message `suspicious_gain` et demande une revue par une seconde personne, qui doit écrire dans la carte du modèle pourquoi le gain est réel. Depuis un an, déclenché trois fois : deux fuites trouvées, un vrai gain (le passage aux quantiles en log pour l'ETA).

## Comment l'appliquer ailleurs

- Toute feature agrégée sur une fenêtre se définit par rapport à la date à laquelle la prédiction sera faite, et cette date est une colonne explicite du jeu d'entraînement.
- Quand un score paraît trop bon, chercher la fuite avant de fêter. Le temps moyen pour trouver une fuite une fois qu'on la cherche : une heure. Le temps perdu quand on ne la cherche pas : deux semaines de production avec un modèle pire que rien.
- Mettre la référence naïve dans chaque évaluation. Un modèle s'évalue contre elle, pas dans l'absolu.
