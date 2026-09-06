---
name: playtest-feedback-loop
description: What four external playtests taught: observation beats the questionnaire, telemetry says where, testers say why, three signals
type: feedback
status: active
verified: 2026-05-20
---

## Ce qu'on collecte

À chaque playtest externe (quatre depuis novembre 2025, 20 à 40 testeurs) :

1. **Télémétrie** : position toutes les 5 s, événements de jeu (accostage, chavirage, menu ouvert), temps par zone, et les rapports de crash ([[crash-triage-lesson]] côté moteur).
2. **Observation** : une personne du studio par groupe de 4 testeurs, qui note sans intervenir. Un formulaire d'observation d'une page : où le testeur hésite, ce qu'il dit à voix haute, où il quitte.
3. **Questionnaire** de fin de session, 12 questions, dont 3 ouvertes.

## Ce qu'on a appris

- **La télémétrie dit où, le testeur dit pourquoi, et ni l'un ni l'autre ne suffit.** Le point de chaleur devant la capitainerie (40 % des testeurs y tournent 2 minutes) n'a été compris qu'avec les notes d'observation : la porte ressemblait à un décor.
- **Le questionnaire surestime la satisfaction.** Note moyenne 7,8/10 au playtest de février, mais 30 % des testeurs n'ont pas atteint la deuxième baie, ce que la télémétrie montrait.
- **Les trois signaux qui ont précédé les plus gros correctifs** : un lieu où plus de 25 % des testeurs restent plus de 90 s sans événement de jeu ; une action tentée trois fois de suite sans succès (tracée par l'événement `input.repeat`) ; et une phrase d'observation qui revient chez trois observateurs différents.
- Les testeurs qui ont déjà joué à un playtest précédent ne servent plus à mesurer la découverte ; on les garde pour la difficulté et on les compte à part.

## Comment appliquer

- Lire les notes d'observation avant la télémétrie, sinon on cherche dans les données ce qu'on croit déjà.
- Une question ouverte de plus vaut mieux qu'une échelle de plus.
- Un correctif issu d'un playtest est vérifié au playtest suivant par le même signal ; le point de chaleur de la capitainerie est passé de 40 % à 6 % après le changement de porte.

Le résultat de chaque playtest est résumé dans le ticket de jalon ([[milestone-process]]).
