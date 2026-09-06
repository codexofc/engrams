---
name: edi-partner-feedback-status-codes
description: Partner EDI teams want fewer unambiguous IFTSTA codes (7, not 14) with the event source in FTX; geofence arrivals needed a SRC marker to be trusted
type: feedback
status: active
verified: 2026-02-19
---

# Ce que les partenaires ont dit de nos statuts

Retours recueillis entre octobre 2025 et février 2026 auprès des équipes EDI et transport de quatre partenaires, par ticket, par visio pour deux d'entre eux. Les statuts sortants sont décrits dans [[edi-status-messages-iftsta]] ; cette note dit ce que les partenaires en ont fait et ce que ça nous a appris.

## Moins de codes, pas plus

Notre premier jeu (2024) proposait 14 codes, dont `13` (départ du dépôt du transporteur), `31` (en douane), `9` (conflit). Les partenaires les ont activés à l'onboarding « pour avoir tout », puis trois sur quatre les ont désactivés dans les six mois. Raisons données :

- Leur TMS n'a qu'une case « statut » par expédition ; un code intermédiaire écrase le précédent et un opérateur qui lit « conflit » sans savoir lequel appelle son transporteur, qui appelle nous.

- Un code qui arrive dans le désordre (l'ETA `20` après le `23` arrivé) rétrograde visuellement l'expédition chez eux.

Ce qu'ils gardent tous : `1`, `21`, `24`, `6`, `23`, `7`, `4`. Sept codes, l'histoire d'un chargement sans ambiguïté. Le jeu par défaut proposé à un nouveau partenaire est passé de 14 à ces 7 en décembre 2025 ; l'ETA et le retard sont proposés à part, avec un avertissement sur le volume.

## « Arrivé » doit dire qui le dit

Quand la détection d'arrivée par géorepérage est arrivée (projet télématique), les codes `24` et `23` ont commencé à partir 10 minutes plus tôt qu'avant, à l'heure détectée plutôt qu'au bouton du chauffeur. Deux partenaires ont d'abord cru à un bug (« votre camion arrive avant d'être arrivé »), parce que leur quai n'avait pas encore vu le camion : il attendait à la barrière, dans le rayon.

Ce qu'ils ont demandé, et qu'on a fait : `FTX+AAO` avec `SRC:GEOFENCE` ou `SRC:DRIVER` sur les statuts d'arrivée et de départ. Leur logistique s'en sert : une arrivée `GEOFENCE` déclenche la préparation du quai, une arrivée `DRIVER` la confirme. Nordkarton a même demandé les deux événements (détection puis confirmation) comme deux IFTSTA `24` successifs ; refusé pour ne pas doubler le volume, mais le marqueur suffit à leur processus.

Leçon : une donnée plus précise que celle d'avant est lue comme fausse si on ne dit pas d'où elle vient.

## L'ETA : utile pour un partenaire sur quatre

Vestaflor (frais, quais à réserver) veut l'ETA à chaque publication et s'en sert. Les trois autres l'ont désactivée : trop de messages, et leur processus n'a rien à en faire tant que le camion n'est pas en retard. La règle de publication « seulement au franchissement de créneau » (projet télématique) est un compromis qu'ils n'ont même pas voulu ; ils veulent le code `17` (retard) avec le motif, et rien d'autre. Ce qu'on en tire : proposer le retard seul par défaut, l'ETA en option.

## Ce qu'ils voudraient et qu'on ne fait pas

- La position du camion dans le statut. Non, pour les raisons du projet conformité, expliquées à chaque fois ; deux partenaires ont compris, un l'a redemandé trois fois.

- Un statut « facturé » avec le montant. Le montant est dans l'INVOIC ([[edi-invoice-invoic-outbound]]) ; un statut avec un prix est une facture déguisée que leur comptabilité ne verrait pas.

- Un renvoi périodique de l'état complet « pour resynchroniser ». Non : un message par événement, et la réconciliation quotidienne ([[edi-reconciliation-daily]]) rattrape les trous. Un des partenaires l'a accepté après avoir vu que le rapport mensuel trouvait ses écarts avant lui.

## Ce que ça a changé

- Jeu par défaut à 7 codes, ETA et retard en option séparée.

- `SRC:` dans `FTX+AAO` pour tous les partenaires, pas seulement ceux qui l'ont demandé.

- La page d'onboarding EDI dit désormais « moins de codes, mieux définis » en premier point, avec les chiffres ci-dessus. Les deux derniers partenaires ont démarré à 7 codes sans en demander plus.
