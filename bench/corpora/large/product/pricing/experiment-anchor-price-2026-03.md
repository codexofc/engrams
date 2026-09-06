---
name: experiment-anchor-price-2026-03
description: Experiment HF-2480 (2026-03-02 to 2026-04-13) showing the suggested price as an anchor in the bid form raised bid clustering around the suggestion from 44 % to 58 % and lowered awarded price by 2.1 % with no loss in fill rate
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
