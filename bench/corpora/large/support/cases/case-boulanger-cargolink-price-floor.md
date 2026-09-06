---
name: case-boulanger-cargolink-price-floor
description: Avril 2026, les baisses de prix de Boulanger Industries refusées par le plancher Cargolink, erreur stockée jamais affichée au chargeur
type: project
status: active
verified: 2026-06-15
---

# Cas : Boulanger Industries, le prix qui ne descendait pas

Chargeur fictif, équipement industriel, Enterprise, publie tout sur Halden et Cargolink. Ticket du 2026-04-14, `integration:partner`, par le responsable transport, qui avait comparé les deux écrans.

## Ce qui s'est passé

Boulanger baissait ses prix indicatifs en cours de journée quand un chargement ne trouvait pas d'offre. Sur Halden, le nouveau prix s'affichait. Sur Cargolink, l'ancien restait. Les transporteurs Cargolink misaient donc sur un prix plus haut que ce que Boulanger voulait payer, Boulanger refusait, et le chargement traînait.

`hfctl partner refs` sur un chargement : `sync_state = PUSH_FAILED`, `last_error = price_below_minimum (lane DE-FR, min 1.32 EUR/km)`. Cargolink a un prix plancher par corridor, contractuel, pour protéger ses transporteurs. Notre push était refusé avec une erreur claire... que personne ne voyait : ni le chargeur (rien dans son écran), ni le support (personne ne regardait `partner refs` sans ticket).

## Ce qu'on a fait

- Explication à Boulanger, avec le plancher du corridor. Ils ne le connaissaient pas ; leur contrat Cargolink le mentionne en annexe.

- HF-3112 : un `PUSH_FAILED` avec une erreur de validation partenaire s'affiche au chargeur sur la fiche du chargement (« Cargolink : prix sous le minimum du corridor, 1.32 EUR/km, l'annonce Cargolink garde l'ancien prix »). Livré mai 2026.

- Le playbook partenaire a gagné la ligne « prix différent : lire `last_error` avant tout ».

- Une requête hebdomadaire sur `partner_load_refs` en `PUSH_FAILED` depuis plus d'une heure, envoyée au support le lundi pour le tri : 40 à 80 lignes par semaine, presque toutes `price_below_minimum` ou `unsupported_vehicle_type`.

## Ce qu'on a appris

- Une erreur stockée et jamais affichée est une erreur perdue. Le champ `last_error` existait depuis le début de l'intégration ; il servait au débogage, pas au client.

- La règle du partenaire n'est pas la nôtre, mais c'est nous qui affichons le résultat. Si le partenaire refuse, on le dit avec ses mots.

- Enterprise, responsable de compte, et pourtant deux semaines d'écart de prix avant le ticket : les clients ne comparent pas leurs écrans tous les jours. L'alerte doit venir de nous.

## Suite

Boulanger a fixé un prix plancher par corridor dans ses propres règles de publication pour ne plus descendre sous celui de Cargolink. Ils ont demandé qu'on l'applique automatiquement (« ne pousse pas sous le plancher, dis-le moi ») : HF-3113, dans le backlog partenaires. Compté dans [[case-lessons-recurring-themes-2026-h1]].
