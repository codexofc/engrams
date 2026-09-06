---
name: playbook-carrier-rating-complaint
description: Contestation de note transporteur : lire la décomposition, requalifier un retrait fait par le chargeur, ce qui ne se discute pas
type: reference
status: active
verified: 2026-06-18
---

# Le transporteur conteste sa note

Catégorie `other:rating` en attendant mieux. La note (sur 5, affichée aux chargeurs) a baissé, le transporteur veut savoir pourquoi et veut qu'on la remonte. On explique, on corrige les faits faux, on ne touche pas au calcul.

## Lire la note

`hfctl carrier rating <carrier_org_id>` : la note courante, la précédente, et la décomposition sur les 90 derniers jours :

- `on_time_pickup` et `on_time_delivery` (part des chargements dans la fenêtre, avec tolérance de 30 minutes)

- `withdrawals` (retraits après acceptation, les plus pénalisants)

- `pod_within_24h` (POD déposé dans les 24 h après livraison)

- `disputes_lost` (litiges tranchés en faveur du chargeur)

- `volume` (le poids de la note dépend du nombre de chargements, sous 10 chargements la note est affichée « nouveau »)

`hfctl carrier rating-events <carrier_org_id> --since 90d` liste les événements qui ont pesé, par chargement.

## Ce qui se corrige

- **Un retrait qui n'était pas un retrait.** Le chargeur a annulé après acceptation et l'événement a été enregistré comme `withdraw_carrier` : c'est visible dans `hfctl load events`, l'acteur est le chargeur. Ça arrive quand le chargeur passe par « retirer le transporteur » puis annule au lieu d'annuler directement. L2 requalifie avec `hfctl rating requalify <event_id> --as shipper_cancel --ticket ... --apply`, la note est recalculée dans la nuit. Macro `rating-requalified`.

- **Une livraison à tort annulée** ([[playbook-undeliver-wrong-delivered]]) : le recalcul est automatique au prochain passage, rien à faire, dire au transporteur d'attendre le lendemain.

- **Un retard dû au chargeur** (quai indisponible, marchandise pas prête) : le chauffeur peut le déclarer dans l'app au moment du pickup (« Attente », avec la durée). S'il l'a fait, le retard est neutralisé automatiquement. S'il ne l'a pas fait, le transporteur demande au chargeur de confirmer dans le ticket ; avec la confirmation écrite du chargeur, L2 requalifie. Sans, non.

- **Un chargement `wrong_pickup`** annulé en transit compte comme retrait, c'est voulu ([[playbook-cancel-in-transit-two-person-rule]]).

## Ce qui ne se discute pas

- Le poids des critères. Il est le même pour tous et documenté dans l'aide transporteur.

- Un retard réel, même de 35 minutes, même « à cause des bouchons ».

- Un litige tranché. La contestation du litige a eu lieu avant, avec le chargeur.

- « Le concurrent a une meilleure note avec moins de chargements ». Sous 10 chargements la note est « nouveau », au-dessus elle reflète les 90 jours.

## Réponse

La macro `rating-explain` prend la décomposition et les trois événements les plus pénalisants et les met en clair. Elle finit par ce que le transporteur peut faire (déclarer les attentes, déposer les POD le jour même). Court, factuel, pas de « malheureusement ».

## Escalade

L2 pour toute requalification. Responsable de compte si un transporteur Enterprise menace de partir ; le support n'a pas de geste commercial à offrir et ne doit pas le laisser croire.

## Pourquoi c'est sensible

La note est visible des chargeurs et pèse sur l'acceptation des offres. Le retrait pénalise fort par choix produit (HF-3170 revoit la pénalité pour les retraits sous 2 h après acceptation, en cours). Un transporteur qui comprend le calcul conteste moins, d'où la décomposition affichée dans son écran depuis mai 2026.
