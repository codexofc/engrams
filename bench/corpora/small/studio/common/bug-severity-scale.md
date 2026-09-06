---
name: bug-severity-scale
description: Bug severity 1 to 4 from blocks the playtest to cosmetic, decides order in week 6 and who may lower it, April 2026 counts
type: reference
status: active
verified: 2026-04-24
---

## Échelle

| Sévérité | Définition | Exemple | Délai |
|---|---|---|---|
| 1 | empêche de jouer ou de construire : crash reproductible, sauvegarde perdue, lane code rouge sur `main` | le cotre traverse le quai | même jour, avant toute autre chose |
| 2 | bloque un objectif du jalon ou une partie du jeu | l'accostage échoue dans une baie | avant le tag du jalon |
| 3 | visible par un testeur, contournable | un PNJ marche dans l'eau | semaine 6 si le temps le permet |
| 4 | cosmétique ou dev uniquement | un avertissement dans l'overlay | quand quelqu'un passe par là |

La sévérité est posée par celui qui ouvre le ticket. Seul le lead du domaine peut la baisser, et il écrit pourquoi. Elle peut être montée par n'importe qui.

## Ordre en semaine 6

Sévérité, puis nombre de rapports (un bug de sévérité 3 signalé par 20 testeurs passe avant un bug de sévérité 3 vu une fois), puis ancienneté. Le tag du jalon ([[milestone-process]]) se fait avec zéro sévérité 1 et 2 ouvertes ; les 3 et 4 sont listées dans le ticket du jalon.

## Avril 2026 (M13)

Ouverts pendant le cycle : 3 de sévérité 1, 14 de sévérité 2, 71 de sévérité 3, 40 de sévérité 4. Fermés avant le tag : tous les 1 et 2, 31 des 3, 6 des 4. Les trois sévérité 1 : le chavirage infini en vagues courtes (physique), le doublon d'objet en annulation entre chunks (éditeur), le crash de la cible basse à la reprise après veille.

## Ce qui n'est pas un bug

Une demande de changement de design est un ticket `feat`, même si le design actuel est mauvais. Un bug est un écart entre ce qui a été décidé et ce qui se passe. Cette distinction a été posée après le playtest de novembre 2025, où 40 « bugs » étaient des désaccords sur la vitesse du cotre.
