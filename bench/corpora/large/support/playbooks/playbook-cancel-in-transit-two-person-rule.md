---
name: playbook-cancel-in-transit-two-person-rule
description: Annulation d'un chargement IN_TRANSIT : support seul, deux personnes, quatre motifs admis, jeton de confirmation hfctl à 10 minutes
type: project
status: active
verified: 2026-06-25
---

# Annuler un chargement en transit

La transition `cancel_in_transit` existe dans la machine à états pour le support uniquement. Elle se fait à deux, avec un ticket HF, et pour quatre motifs. Tout ce qui n'est pas dans la liste est un « non », voir [[playbook-shipper-cannot-cancel-load]].

## Les motifs admis

`hfctl load cancel-reasons --in-transit` les liste :

- `accident` : le véhicule est accidenté, la marchandise ne sera pas livrée par ce transporteur. Pièce attendue : déclaration du transporteur dans le ticket.

- `goods_destroyed` : la marchandise est perdue ou détruite (incendie, vol). Pièce : déclaration du transporteur, et le chargeur prévenu.

- `shipper_vanished` : le chargeur est injoignable, le destinataire refuse la livraison, le transporteur ramène la marchandise. Rare (deux cas en 2026). Pièce : trois tentatives de contact tracées.

- `wrong_pickup` : le chauffeur a fait `pickup` sur le mauvais chargement, le camion est vide de cette marchandise-là. Le motif le plus fréquent. Pièce : confirmation du transporteur, et le bon chargement identifié.

Pas de motif « changement d'avis du chargeur », « retard », « prix contesté ». Ces cas se règlent entre chargeur et transporteur avec une livraison puis un nouveau chargement.

## Procédure

1. Ticket Deskline avec le motif, les pièces, `load_id`. L2 ouvre un ticket HF (`HF-` avec le préfixe `[cancel-in-transit]`) et y colle le tout.

2. `hfctl load cancel-in-transit <load_id> --reason wrong_pickup --ticket HF-3xxx` (sans `--apply`) : affiche l'état, le chauffeur, les positions récentes, et ce qui va se passer.

3. Avec `--apply` : la commande imprime un jeton de confirmation à six caractères et attend. Un second L2, ou le backend d'astreinte, tape `hfctl confirm <jeton>` depuis sa propre session dans les 10 minutes. Sinon la commande expire, rien n'est fait.

4. La transition écrit un `load_events` de type `cancel_in_transit` avec les deux acteurs et le ticket. Le chargeur et le transporteur reçoivent la notification d'annulation, l'intégrateur reçoit `load.cancelled` avec `reason`.

5. Le support répond aux deux parties avec la macro `cancel-in-transit-done`.

## Après

- **Facturation** : un chargement annulé en transit ne passe jamais en `INVOICED`. Les frais éventuels (transport à vide, retour) font l'objet d'une facture manuelle par la finance à la demande du transporteur, hors plateforme. Le dire dans la réponse.

- **Note du transporteur** : `accident` et `goods_destroyed` n'affectent pas la note. `wrong_pickup` compte comme un retrait. `shipper_vanished` n'affecte pas le transporteur.

- **Le bon chargement** dans le cas `wrong_pickup` : le chauffeur fait `pickup` dessus, en retard, avec l'horodatage réel. On n'antidate pas.

## Pourquoi deux personnes

Avant HF-3075 (février 2026) un seul L2 pouvait le faire. En janvier 2026 un chargement a été annulé en transit sur la foi d'un e-mail qui venait d'un chargeur homonyme. Le transporteur a livré quand même, la facture n'a pas pu être émise par la plateforme, et il a fallu trois semaines de compta manuelle. Le jeton à deux n'empêche pas l'erreur, il force une seconde lecture.

## Ce qu'on ne fait pas

Pas d'annulation en transit le vendredi après 16 h sauf `accident` ou `goods_destroyed`. Pas d'annulation sur demande orale. Pas de contournement du jeton par la même personne avec deux sessions (c'est journalisé, et c'est arrivé une fois, la personne a été rappelée à l'ordre).

## Exemple de ticket HF complet

Le ticket HF pour une annulation en transit suit le gabarit `[cancel-in-transit]` et tient en une page. Ce qu'il contient, dans l'ordre, tiré d'un cas `wrong_pickup` de mai 2026 (identifiants remplacés) :

- Titre : `[cancel-in-transit] load <ref> · wrong_pickup · Deskline #<id>`

- Contexte : « Le chauffeur a fait pickup sur le chargement <ref A> à 07:42 alors que la marchandise chargée est celle de <ref B>, même quai, même chargeur. Le transporteur le confirme dans Deskline (message du 2026-05-14 08:10). Le chargeur est prévenu. »

- Sortie de `hfctl load get <A>` et `hfctl load events <A>` collée telle quelle.

- Sortie du dry run de `cancel-in-transit`.

- Ligne « Second : <login L2> » ajoutée par la seconde personne après `hfctl confirm`.

- Sortie de `--apply` avec l'identifiant d'audit.

- Suite : « <ref B> : pickup à refaire par le chauffeur, transporteur prévenu. Facturation : rien à émettre sur <A>. »

Le ticket est fermé le jour même par la première personne. Il n'est pas rattaché à un epic ; les annulations en transit se retrouvent par le préfixe du titre.

## Chiffres depuis HF-3075

Février à juillet 2026 : 11 annulations en transit, 9 `wrong_pickup`, 1 `accident`, 1 `goods_destroyed`. Aucun `shipper_vanished`. Délai médian entre la demande Deskline et l'`--apply` : 1 h 50 en journée. Deux demandes refusées parce que le motif réel était un changement d'avis du chargeur, toutes deux réglées par une livraison puis un nouveau chargement. Une tentative de contournement du jeton par la même personne avec deux sessions, détectée par l'audit (même login, même minute), sans conséquence sur le chargement, avec un rappel de la règle.

## Ce que voit l'intégrateur

Le `load.cancelled` émis porte `reason` (le code du motif) et, en v3, `meta.ticket` avec la clé HF. Les intégrateurs qui modélisent la responsabilité de la marchandise lisent `reason` : `accident` et `goods_destroyed` déclenchent chez eux un dossier sinistre, `wrong_pickup` rien. C'est documenté dans le guide intégrateur depuis avril 2026 à la demande d'un chargeur Enterprise dont l'assureur voulait la distinction.
