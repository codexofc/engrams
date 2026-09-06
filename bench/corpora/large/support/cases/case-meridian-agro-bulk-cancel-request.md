---
name: case-meridian-agro-bulk-cancel-request
description: Octobre 2025, Meridian Agro demande d'annuler « tous les chargements de la semaine prochaine », 17 annulés à tort, règle : pas d'annulation en masse
type: project
status: active
verified: 2026-01-15
---

# Cas : Meridian Agro, l'annulation en masse qu'on n'aurait pas dû faire

Chargeur fictif, agroalimentaire, Business, quatre sites. Ticket du 2025-10-22, catégorie `load:cancel`, à l'époque où le support pouvait encore annuler pour le client.

## Ce qui s'est passé

Un responsable logistique écrit : « merci d'annuler tous nos chargements de la semaine prochaine, grève à l'usine ». L2 liste les chargements de l'org en `OPEN`, `BIDDING` et `DISPATCHED` avec pickup entre le 27 et le 31 octobre : 23. Il les annule, avec le motif `shipper_request`, un par un, en quarante minutes.

Le lendemain, trois autres personnes de Meridian ouvrent des tickets : leurs chargements ont disparu. La grève concernait **un** site sur quatre. Le responsable qui avait écrit gérait ce site et parlait de « nos chargements » au sens de son équipe. 17 chargements annulés à tort, dont 9 `DISPATCHED` : neuf transporteurs prévenus d'une annulation, dont quatre qui avaient déjà refusé d'autres chargements pour tenir ceux-là.

## Ce qu'on a fait

- Les chargements `CANCELLED` ne se réactivent pas. Meridian a republié 17 chargements ; 6 ont été repris par les mêmes transporteurs, les autres ont trouvé preneur à un prix plus haut en moyenne de 8 %.

- Les frais d'annulation `DISPATCHED` ont été annulés côté plateforme par la finance (avoir) puisque l'erreur était la nôtre au moins autant que la leur. Les transporteurs ont été dédommagés par Meridian directement pour les quatre cas de refus de chargements tiers.

- Excuses écrites du responsable support, au responsable de compte et au client.

## Ce qu'on a changé

- **Le support n'annule pas en masse.** Point. Le client a la sélection multiple dans sa liste depuis HF-3020 (novembre 2025, Business et au-dessus), avec un récapitulatif par site avant confirmation. C'est écrit dans le playbook d'annulation.

- Une annulation par le support (unitaire, L2, sur demande écrite) exige l'identifiant du chargement, pas une description. « Tous ceux de la semaine prochaine » n'est pas une demande recevable.

- `hfctl load cancel` refuse une liste de plus de cinq identifiants depuis 1.9.

## Ce qu'on a appris

- « Nos » ne veut jamais dire ce qu'on croit. Le périmètre d'une demande est celui de la personne qui écrit, pas celui de l'organisation.

- Faire pour le client est confortable pour lui et pour nous jusqu'au jour où c'est faux. Le client qui clique lui-même voit le récapitulatif. Nous, on ne voyait qu'une liste.

- Quarante minutes d'exécution sans une seule question posée. Une demande qui touche plus de cinq objets mérite une confirmation écrite du périmètre avant d'agir. C'est la première règle de [[case-lessons-recurring-themes-2026-h1]].

## Suite

Meridian est resté client. Le responsable logistique, un peu gêné, a demandé une fonction de « pause par site » qui suspend les publications d'un site sans annuler ce qui est déjà dispatché. Livrée en HF-3090 (mars 2026), utilisée par onze chargeurs depuis.
