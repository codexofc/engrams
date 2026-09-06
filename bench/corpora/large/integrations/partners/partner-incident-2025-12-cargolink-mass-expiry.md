---
name: partner-incident-2025-12-cargolink-mass-expiry
description: Décembre 2025, 640 annonces Cargolink expirées à 14 jours recréées avec de nouveaux identifiants, 212 offres perdues en 404, d'où previous_external_ids
type: project
status: active
verified: 2026-01-30
---

# Incident de décembre 2025 : les annonces expirées de Cargolink (HF-3065)

## Ce qui s'est passé

Cargolink supprime une annonce 14 jours après sa création. Entre le 20 décembre et le 2 janvier, beaucoup de chargements restent `OPEN` longtemps : les chargeurs publient pour janvier, les transporteurs sont en congé, rien ne bouge. Le 27 décembre, environ 640 annonces poussées entre le 10 et le 13 décembre ont expiré chez Cargolink le même week-end.

La synchro de l'époque ([[partner-cargolink-sync-polling-v1]]) comparait nos chargements actifs avec leurs annonces actives. Annonce disparue, chargement toujours `OPEN` : « créer ». Elle a recréé les 640 annonces, avec 640 nouveaux identifiants externes, et écrasé `external_id` dans `partner_load_refs`. L'ancien identifiant n'était gardé nulle part.

Les transporteurs Cargolink qui avaient mis en favori ou consulté les anciennes annonces ont continué à miser dessus dans leur interface pendant quelques jours (Cargolink garde une annonce expirée consultable 7 jours, avec la possibilité de faire une offre « hors délai » que le propriétaire peut accepter). Leurs offres arrivaient sur notre callback avec l'ancien identifiant. Notre résolveur ne le connaissait plus : 404. Cargolink, en 404, arrête de réessayer. 212 offres perdues entre le 27 décembre et le 1er janvier, vues le 2 janvier dans leurs logs quand un chargeur a demandé pourquoi un transporteur disait avoir misé.

## Ce qu'on a fait

- 2 janvier : Cargolink nous a envoyé la liste des 212 offres avec l'ancien identifiant. On a reconstitué la correspondance ancien/nouveau par `load_id` (heureusement présent dans leur champ d'identifiant externe) et réinjecté les offres encore pertinentes (le chargement toujours `OPEN`, 140 offres). Les chargeurs ont été prévenus par le support.

- 72 offres concernaient des chargements entre-temps attribués ailleurs ; rien à faire, sauf s'excuser auprès de Cargolink pour leurs transporteurs.

## Correctifs

- `previous_external_ids[]` dans `partner_load_refs` ([[partner-dedup-external-refs-table]]) et un résolveur qui cherche aussi dedans. Livré le 9 janvier.

- Le re-listing explicite avant expiration (13 j 12 h) avec le champ `previous_listing_id` de Cargolink, qui rattache les favoris et les offres de l'ancienne annonce à la nouvelle de leur côté. Livré avec la synchro événementielle ([[partner-cargolink-sync-push-feed]]) en mars ; entre janvier et mars, le re-listing était fait par la comparaison v1 modifiée pour appeler `relist` au lieu de `create` quand un `external_id` existait.

- Le callback répond 409 `listing_superseded` avec le nouvel identifiant au lieu de 404 quand l'ancien est connu, ce que Cargolink traite en réémettant sur le nouveau.

- Une alerte si plus de 100 annonces expirent le même jour (`hf_partner_listings_expired_total`), pour voir venir la prochaine période de fêtes.

## Ce qu'on a appris

- Un identifiant externe qu'on écrase est un identifiant qu'on perd. Garder l'historique coûte un tableau.

- Une règle contractuelle (14 jours) est une règle du code. Elle était dans le contrat et pas dans le code.

- La période de fêtes est un cas de test : des chargements qui restent ouverts trois semaines, ce qui n'arrive jamais en mars. Le jeu de test de la synchro a maintenant un scénario « annonce de 20 jours ».

- Un 404 est une réponse définitive pour un partenaire. Répondre 404 à quelque chose qu'on a peut-être connu est une perte silencieuse. Le résolveur ne répond plus 404 qu'après avoir cherché dans l'historique.

## Chiffres

640 annonces, 212 offres, 140 réinjectées, 5 jours. Aucun chargement perdu, quelques chargeurs qui ont dispatché plus cher qu'ils n'auraient pu. Cargolink a été compréhensif ; c'était leur premier partenaire à leur signaler que les offres hors délai arrivaient avec l'ancien identifiant, et ils ont ajouté le nouveau dans leur payload depuis.
