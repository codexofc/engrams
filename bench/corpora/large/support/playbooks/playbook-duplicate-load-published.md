---
name: playbook-duplicate-load-published
description: Chargement publié en double : double soumission, doublon partenaire Fretzone ou Cargolink, ou flux répété, et qui annule quoi
type: project
status: active
verified: 2026-06-25
---

# Chargement publié en double

Catégorie `load:duplicate`. Deux chargements identiques visibles par les transporteurs, deux séries d'offres, et un transporteur qui va accepter le mauvais. Il faut agir vite mais c'est au chargeur d'annuler.

## Distinguer les trois cas

1. `hfctl load list --org <org_id> --reference <ref>` (la référence du chargeur, champ `reference`). Deux résultats créés à moins d'une minute d'écart avec les mêmes adresses et fenêtres : **double soumission** du formulaire web. Depuis HF-3092 le formulaire porte une clé d'idempotence et ça ne devrait plus arriver depuis le web ; ça arrive encore par API quand l'intégrateur rejoue un `POST /v2/loads` sans `Idempotency-Key`.

2. Un des deux porte `source = fretzone` ou `source = cargolink` dans `hfctl partner refs <load_id>` : **doublon partenaire**. Le chargeur a publié chez nous et sur le partenaire, et notre synchro a rapatrié sa propre annonce. Ce cas a son propre traitement plus bas.

3. Références différentes, dates différentes : ce n'est pas un doublon, c'est un flux hebdomadaire. Ne rien faire, macro `load-not-duplicate`.

## Double soumission

- Si aucun des deux n'a d'offre : le chargeur annule celui qu'il veut depuis son écran (`cancel` depuis `OPEN`). Macro `load-duplicate-cancel-yourself`. On ne choisit pas pour lui.

- Si un seul a des offres : il garde celui-là et annule l'autre. Même macro, on lui dit lequel.

- Si les deux ont des offres, ou si l'un est déjà `DISPATCHED` : le chargeur annule l'autre, et les transporteurs qui avaient misé dessus reçoivent la notification d'annulation. Un transporteur qui avait été accepté sur le doublon annulé est en droit de réclamer les frais d'annulation du contrat. Prévenir le responsable de compte si c'est un Enterprise.

- Intégrateur API : macro `api-idempotency-key` avec le lien de la doc. Le ticket est tagué pour le tri, on compte ces cas.

## Doublon partenaire

Le chargement rapatrié depuis Fretzone ou Cargolink a `visibility = PARTNER_ONLY` en sens inverse : c'est un chargement du partenaire affiché chez nous. La synchro le retire quand le partenaire le retire. Le chargeur doit supprimer son annonce chez le partenaire ou chez nous, pas les deux.

Depuis HF-3210 (en cours, juillet 2026) la synchro détecte les doublons par `(org_id, reference, pickup_date)` et n'importe pas une annonce qui correspond à un chargement direct du même chargeur. En attendant : `hfctl partner unlink <load_id> --apply` (L2) retire le chargement rapatrié de notre recherche sans toucher au partenaire.

## Ce qu'on ne fait pas

Le support n'annule pas un chargement en double à la place du chargeur, sauf demande écrite dans le ticket et absence d'offre, et alors c'est L2 avec `hfctl load cancel --reason duplicate`. On ne fusionne pas deux chargements, ça n'existe pas.

## Pourquoi c'est un projet

Le volume a justifié HF-3092 (clé d'idempotence du formulaire) et HF-3210 (détection à la synchro). Les cas restants sont API sans `Idempotency-Key`, et le tri hebdomadaire regarde si une politique plus dure (refuser le POST sans clé) se justifie. Voir aussi [[playbook-partner-load-mismatch]].
