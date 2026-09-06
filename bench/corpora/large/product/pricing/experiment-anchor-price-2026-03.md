---
name: experiment-anchor-price-2026-03
description: HF-2480: pre-filling the suggested price in the bid form raised clustering from 44 % to 58 % and cut awarded price 2.1 %
type: project
status: active
verified: 2026-04-20
---

## Question

Le formulaire d'enchère affichait le prix suggéré en petit sous le champ montant. On voulait savoir si l'afficher comme valeur pré-remplie (ancrage) changeait la distribution des enchères et le prix final payé par le chargeur.

## Dispositif

- Flag `pricing.bid_form_anchor`, répartition 50/50 par `carrier_id` (hash stable, pas par session, pour qu'un transporteur voie toujours la même chose). Convention des flags dans les notes communes produit.
- Période : 2026-03-02 au 2026-04-13, six semaines, entités FR et DE seulement (PL exclu à cause du changement KSeF en cours).
- Métriques primaires : écart relatif entre l'enchère et la suggestion, prix attribué relatif à la suggestion. Secondaires : taux de remplissage des chargements (au moins une enchère), taux d'attribution, délai à la première enchère.
- Analyse sur les enchères `open` initiales, pas sur les ré-enchères, en suivant les règles de [[pricing-experiment-guidelines]].

## Résultats

Sur 61 300 enchères en A (petit texte) et 60 900 en B (pré-rempli) :

| Métrique | A | B |
|---|---|---|
| enchères à ±5 % de la suggestion | 44,1 % | 58,3 % |
| enchères exactement égales à la suggestion | 3,2 % | 21,7 % |
| prix attribué / suggestion (médiane) | 1,038 | 1,017 |
| taux de remplissage | 93,4 % | 93,6 % |
| taux d'attribution | 70,8 % | 71,4 % |
| délai médian première enchère | 11,5 min | 9,8 min |

Le prix attribué baisse de 2,1 % en relatif, sans perte de remplissage ni d'attribution. Les transporteurs enchérissent plus vite, sans doute parce que le formulaire demande moins de réflexion.

## Ce qu'on a regardé de plus près

- Les petites flottes (1 à 3 camions) ancrent beaucoup plus (31 % d'enchères exactement égales) que les flottes de plus de 20 camions (6 %). Le gain de prix vient d'elles.
- Aucun signe que le taux de litige post-livraison bouge (2,0 % contre 2,1 %).
- Sur les lignes où la suggestion était mauvaise (clusters à faible confiance, voir [[pricing-lane-clusters]]), B augmente la dispersion au lieu de la réduire : les transporteurs corrigent une ancre absurde et l'écart-type monte de 18 %. On a décidé de n'afficher l'ancre que si la confiance du cluster est `medium` ou `high`.

## Décision

Déployé à 100 % le 2026-04-20 avec la condition de confiance. Le flag est supprimé du code (HF-2531). Le suivi mensuel de « prix attribué / suggestion » est dans le tableau de bord pricing ; s'il repasse au-dessus de 1,03 sur un mois, on rouvre la question.

## Détails de mise en œuvre

- Le flag `pricing.bid_form_anchor` était de type `experiment` avec deux variantes, `control` et `anchor`, évalué côté serveur dans `GET /api/v1/loads/{id}/quote`, qui renvoyait `ui.prefill_amount` rempli ou nul. Le client mobile et le client web n'avaient aucune logique : ils pré-remplissent si le champ est là. C'est ce qui a permis de lancer l'expérience sans sortie d'app mobile.
- Les enchères des deux variantes sont identifiables dans l'entrepôt par la propriété `experiments` de l'événement `bid.placed` (`pricing.bid_form_anchor:anchor`), et la table `marts.pricing_experiments` a une ligne par enchère initiale avec la variante, le cluster, la taille de flotte et l'écart à la suggestion.
- Requête d'analyse principale, conservée dans `analyses/pricing/anchor-2026-03.sql`, qui prend 3 s sur l'entrepôt.

## Ce qui n'a pas été mesuré et qu'on aurait voulu

- Le comportement des transporteurs après la fin de l'expérience : un transporteur passé de `anchor` à `control` au déploiement à 100 % n'existe pas (tout le monde est passé en `anchor`), mais on n'a pas suivi si les transporteurs `control` ont changé leur façon d'enchérir en découvrant l'ancre. Le prix attribué relatif est passé de 1,017 en B pendant l'expérience à 1,021 en mai pour tout le monde, ce qui suggère un léger effet de nouveauté, ou rien du tout.
- L'effet sur les enchères à prix rond : en B, 21,7 % des enchères égalent la suggestion, mais on n'a pas regardé si les enchères non égales à la suggestion sont devenues moins rondes (moins de 1 500 EUR, plus de 1 486 EUR). Une lecture rapide en juin montre 31 % d'enchères multiples de 50 EUR contre 44 % avant, ce qui dit que les transporteurs ajustent à partir de l'ancre plutôt que de partir d'un chiffre rond.

## Effet de bord sur le plafond chargeur

Les chargeurs voient la suggestion à la publication ([[shipper-price-cap-rule]]) et l'ancre n'a rien changé pour eux, mais la part de chargements avec un plafond sous la suggestion est passée de 11 % à 9 % pendant l'expérience, sans qu'on sache pourquoi : les chargeurs ne voyaient pas la variante. Hypothèse notée : avec des enchères plus proches de la suggestion, les chargeurs qui posent un plafond agressif reçoivent moins d'enchères et apprennent. Non vérifié.

## Suivi

Le tableau de bord pricing affiche « prix attribué / suggestion » par semaine, par entité et par taille de flotte, avec la ligne 1,03 qui rouvrirait la question. Juin 2026 : 1,021 global, 1,012 flottes de 1 à 3 camions, 1,034 flottes de plus de 20. Le seuil est global ; la valeur des grandes flottes au-dessus de 1,03 est connue et acceptée, elles n'ancrent pas et n'ancraient pas avant.
