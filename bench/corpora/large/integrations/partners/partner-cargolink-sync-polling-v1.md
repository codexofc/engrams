---
name: partner-cargolink-sync-polling-v1
description: Première synchro Cargolink (oct. 2025 à mars 2026) : comparaison complète toutes les 2 minutes, mises à jour fantômes, remplacée par le flux événementiel
type: project
status: archived
superseded_by: [[partner-cargolink-sync-push-feed]]
verified: 2025-12-15
---

# Synchronisation Cargolink par comparaison périodique (v1)

Le mécanisme livré avec l'intégration en HF-3015, remplacé en mars 2026 par [[partner-cargolink-sync-push-feed]]. Gardé pour comprendre les tickets et l'incident de décembre 2025.

## Principe

Un worker `app:partners:cargolink:sync` toutes les 2 minutes :

1. Charge tous les chargements `OPEN` ou `BIDDING` des organisations ayant Cargolink activé (entre 3 000 et 6 000 lignes selon l'heure).

2. Charge toutes les annonces actives de notre compte chez Cargolink (`GET /v3/listings?owner=us`, paginé par 200).

3. Compare : chargement sans annonce, créer ; annonce sans chargement actif, supprimer ; les deux mais des champs différents (prix, dates, adresses), mettre à jour.

4. Écrit le résultat dans `partner_load_refs`.

## Ce qui n'allait pas

- **Latence** : jusqu'à 2 minutes plus la durée du cycle (40 à 90 s) pour qu'une baisse de prix arrive chez Cargolink. Les chargeurs qui ajustent leurs prix en journée s'en plaignaient.

- **Charge** : 30 appels paginés chez Cargolink toutes les 2 minutes pour ne rien changer 95 % du temps. Cargolink a demandé en novembre de réduire, on est passé à 3 minutes, ce qui a empiré la latence.

- **La comparaison mentait** sur les champs qu'ils normalisent : on envoyait « Straße », ils renvoyaient « Strasse », le worker voyait une différence et mettait à jour à chaque cycle. 8 % des annonces étaient « mises à jour » à chaque passage sans changement réel, ce qui apparaissait chez eux comme des annonces « modifiées à l'instant » et les remontait dans leur tri, ce que leurs transporteurs ont remarqué et ce qui a fait l'objet d'une remarque contractuelle.

- **L'expiration à 14 jours** n'était pas prise en compte : une annonce expirée chez eux disparaissait de leur liste, le worker la recréait comme neuve, avec un nouvel identifiant externe, sans lien avec les offres reçues sur l'ancienne. C'est [[partner-incident-2025-12-cargolink-mass-expiry]].

## Ce qui a été gardé

La table `partner_load_refs` et sa sémantique ([[partner-dedup-external-refs-table]]), la correspondance des champs ([[partner-load-field-mapping-rules]]), et la comparaison complète elle-même, qui tourne encore une fois par nuit comme filet de sécurité dans la réconciliation.
