---
name: support-findings-onboarding-2026-04
description: What 96 calls to carriers who stalled before the licence upload said in April 2026, three real reasons, product changes made
type: feedback
status: active
verified: 2026-05-02
---

En avril 2026, l'équipe onboarding a appelé 140 transporteurs inscrits en mars, société validée, sans licence envoyée après 10 jours (la plus grosse perte de l'entonnoir, voir [[activation-funnel-q1-2026]]). 96 ont répondu. Ce qu'ils ont dit, regroupé.

## Les trois vraies raisons

1. **« Je ne trouve pas le document » (41 %).** La licence communautaire est un original conservé au siège et des copies conformes dans chaque camion. Le gérant d'une petite flotte n'a pas le document sous la main quand il s'inscrit depuis son téléphone le soir. Il remet à plus tard, puis oublie. Ce n'est pas un problème de motivation ni d'explication.
2. **« Je pensais que la société validée suffisait » (27 %).** Le message de fin d'étape 2 disait « Votre société est vérifiée ». Ils ont compris « vous pouvez travailler » et ont attendu des chargements. L'email du jour 2 de la séquence ([[onboarding-email-sequence]]) le disait, mais 60 % ne l'avaient pas ouvert.
3. **« Je voulais d'abord voir s'il y a des chargements pour moi » (22 %).** Ils ont regardé la bourse, trouvé peu de chargements sur leurs lignes (surtout des transporteurs roumains sur des lignes vers l'Espagne), et n'ont pas vu l'intérêt de continuer.

Le reste : problèmes techniques d'envoi (6 %), déjà inscrits sur une autre bourse et en test (4 %).

## Ce qu'on en tire

- La raison 1 se traite par le produit, pas par les relances : permettre la photo de la copie conforme qui est dans le camion (elle porte le même numéro, on la vérifie de la même façon dans [[licence-community-check]]) et le dire explicitement dans l'écran d'envoi. Changement déployé le 2026-04-24 (HF-2566) : « Vous pouvez photographier la copie conforme qui se trouve dans votre camion. » Envois de licence à J+3 passés de 58 % à 66 % en mai.

- La raison 2 est un problème de formulation. Le message de fin d'étape 2 dit maintenant « Société vérifiée. Il reste votre licence de transport pour pouvoir enchérir. » avec le bouton d'envoi. On ne compte pas sur l'email.

- La raison 3 est réelle et on ne va pas la masquer : un transporteur qui n'a pas de chargements sur ses lignes n'a pas de raison de finir. L'expérience de nudge sur le premier chargement ([[first-load-nudge-experiment]]) montre les chargements ouverts sur ses lignes avant même l'envoi de la licence, pour que la raison 3 devienne un choix informé.

## Comment appliquer ailleurs

Appeler les gens qui décrochent, tous les trimestres, 100 à 150 appels, avant de toucher aux emails. Les trois raisons ci-dessus ne figuraient dans aucune hypothèse de l'équipe avant les appels ; on pensait à un problème de confiance dans la plateforme, qui n'est ressorti que deux fois sur 96.
