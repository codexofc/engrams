---
name: case-sandoval-shared-phone-two-drivers
description: Mai 2026, deux chauffeurs de Sandoval Cargo sur un seul compte, trajets impossibles et note en baisse ; règle « un numéro, un chauffeur »
type: project
status: active
verified: 2026-07-01
---

# Cas : Sandoval Cargo, deux chauffeurs, un numéro

Transporteur fictif espagnol, 8 camions, Starter. Tickets des 2026-05-05 et 05-12, `driver:sync` puis `other:rating`.

## Ce qui s'est passé

Un père et son fils conduisaient en alternance sur la même ligne, avec le même téléphone d'entreprise et donc le même compte chauffeur (un numéro, un PIN, tous les deux le connaissaient). Pour la plateforme, un seul chauffeur livrait douze chargements par semaine, dont certains à 600 km d'écart à trois heures d'intervalle. Le calcul de ponctualité voyait des trajets impossibles et le filtre de positions aberrantes rejetait la moitié des points GPS comme du bruit.

Le premier ticket disait « le suivi GPS ne marche pas ». Le second, une semaine plus tard, « notre note a baissé sans raison ». Les deux venaient du même problème.

## Ce qu'on a trouvé

L2, en lisant `hfctl load positions` sur deux chargements du même jour : deux traces qui s'éloignent l'une de l'autre à 90 km/h. Un chauffeur ne fait pas ça. Appel au gérant, qui a expliqué sans embarras : « c'est mon fils et moi ».

## Ce qu'on a dit

Que le compte chauffeur est nominatif, pas par téléphone ni par camion, pour trois raisons qu'on a écrites : la responsabilité sur la marchandise (le POD porte le nom du chauffeur), le temps de conduite (des chargeurs nous demandent les heures), et le calcul de la note qui suppose une personne par compte. Deux comptes, deux numéros, ou un téléphone à deux cartes SIM : le gérant a pris une seconde carte.

## Ce qu'on a changé

- Les conditions transporteur disent depuis juin 2026, en clair, « un numéro de téléphone correspond à un chauffeur et un seul ». C'était implicite.

- Le playbook chauffeur a une ligne : « deux traces GPS simultanées éloignées = deux personnes sur un compte », avec la commande.

- La note de Sandoval a été recalculée après création du second compte, en réattribuant les chargements passés au bon chauffeur sur déclaration du gérant, ce que le backend a fait par ticket HF-3145 une fois, sans commande, en disant que ce ne serait pas répété.

- Un contrôle hebdomadaire dans l'entrepôt (`marts.driver_impossible_trips`) signale les comptes chauffeur avec deux positions à plus de 200 km d'écart en moins d'une heure. Quatorze comptes trouvés au premier passage, sur 9 000 chauffeurs actifs. Les transporteurs concernés ont été contactés par le support, sans pénalité.

## Ce qu'on a appris

- Un ticket « le GPS ne marche pas » et un ticket « la note baisse » une semaine plus tard, même organisation, c'est un seul ticket.

- Chez les petits transporteurs, le compte est vu comme un outil du camion. La règle nominative n'est pas évidente pour eux et il faut la dire lors de l'inscription, pas après. Le formulaire de création de chauffeur la rappelle depuis juin.

Dans [[case-lessons-recurring-themes-2026-h1]], ce cas est celui de la « donnée impossible » qu'il faut aller chercher.
