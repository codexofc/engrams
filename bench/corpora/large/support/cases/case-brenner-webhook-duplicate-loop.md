---
name: case-brenner-webhook-duplicate-loop
description: Janvier 2026, l'intégrateur de Brenner Spedition répondait 200 sans traiter, puis dédupliquait sur l'heure et non sur X-Halden-Delivery
type: project
status: active
verified: 2026-04-02
---

# Cas : Brenner Spedition, rejeux et doublons

Transporteur fictif autrichien, 60 camions, TMS maison intégré par un prestataire. Business, tickets du 2026-01-06 au 2026-01-20, catégorie `integration:webhook`.

## Ce qui s'est passé

Le prestataire de Brenner recevait nos webhooks `load.dispatched` et `load.delivered`. Chaque matin, le dispatcheur constatait des chargements manquants dans le TMS et demandait un rejeu de la journée précédente. L2 faisait `hfctl webhooks replay-range` sur 24 h. Le lendemain, le TMS avait des doublons, et le prestataire demandait « pourquoi vous envoyez deux fois ».

Le premier L2 a rejoué trois fois en une semaine avant que quelqu'un regarde les livraisons : **toutes étaient `DELIVERED` du premier coup**. Nous n'avions rien perdu. Le TMS ignorait certains événements à la réception (une exception avalée dans leur handler quand le champ `driver` était `null`, ce qui arrive avant assignation du chauffeur) tout en répondant 200. Et à la réception du rejeu, il dédupliquait sur `(load_id, event_type, heure arrondie à la minute)` : le rejeu portant une heure différente, il créait une seconde ligne.

## Ce qu'on a fait

- Arrêt des rejeux le 13. Le playbook a gagné sa règle : on ne rejoue pas une livraison `DELIVERED` sans avoir vu l'erreur côté client.

- Envoi au prestataire d'un export des livraisons avec `X-Halden-Delivery`, corps et code de réponse, sur la semaine. Ils ont trouvé leur exception en une heure.

- Rappel du guide intégrateur : la déduplication se fait sur `X-Halden-Delivery`, qui est stable au rejeu, et `driver` peut être `null` sur `load.dispatched`.

- HF-3060 : `hfctl webhooks deliveries` affiche maintenant le temps de réponse du client et la taille de la réponse ; un 200 en 3 ms avec un corps vide est suspect et le playbook le dit.

## Ce qu'on a appris

- Un 200 n'est pas une preuve de traitement. C'est pour ça que la doc dit de répondre 200 seulement après persistance, mais on ne peut pas le vérifier.

- Le rejeu est un outil de réparation, pas de diagnostic. Trois rejeux sans lire une seule `last_error`, c'est trois occasions manquées de voir que tout était livré.

- Le prestataire n'était pas notre client ; Brenner l'était. La communication technique passait par le dispatcheur, qui traduisait mal dans les deux sens. Depuis, pour un Business ou Enterprise avec un prestataire, on demande un contact technique direct dès le premier ticket d'intégration. C'est dans [[case-lessons-recurring-themes-2026-h1]].

## Suite

Brenner n'a plus ouvert de ticket d'intégration depuis février 2026. Le prestataire a demandé un environnement de test avec des webhooks vers son bac à sable, ce qui existe déjà (`staging_allowed` sur l'abonnement) et qu'il ne connaissait pas. Ajouté au guide intégrateur en première page.
