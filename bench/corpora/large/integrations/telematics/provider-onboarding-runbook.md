---
name: provider-onboarding-runbook
description: Connecting a telematics provider takes 8 steps and about 6 weeks: DPA first, adapter, harness fixtures, a 2 week shadow run on 20 vehicles, then sign-off
type: reference
status: active
verified: 2026-04-20
---

# Runbook : intégrer un nouveau fournisseur télématique

Écrit après Geolyx (2025) et un troisième candidat abandonné en 2025 à l'étape 1. Six semaines quand tout va bien, dont deux d'attente. Les décisions structurelles sont dans [[telematics-providers-overview]].

## Étapes

**1. Conformité avant tout code.** Localisation du traitement, rétention chez eux, usage secondaire, sous-sous-traitants, délai de notification. La liste et les réponses exigées sont dans la note sur les sous-traitants télématiques du projet conformité. Un fournisseur qui ne répond pas « Union européenne, 90 jours maximum, aucun usage secondaire » s'arrête ici. Le troisième candidat de 2025 s'est arrêté ici. Durée : 2 semaines d'échanges.

**2. Contrat et coûts.** Prix par véhicule actif, définition d'« actif », engagement, clause de rejeu et de limite de débit (on demande depuis [[incident-2025-11-geolyx-duplicate-flood]] qu'un rejeu soit signalé par un en-tête). Chiffres de référence dans [[telematics-costs-per-provider]].

**3. Accès de test.** Un compte bac à sable avec au moins 5 véhicules simulés qui bougent. S'ils n'en ont pas, on demande 5 vrais véhicules d'un transporteur volontaire, avec son accord écrit. Secrets dans le vault sous `telematics/<provider>/sandbox`.

**4. Adaptateur.** Une classe `<Provider>Adapter` dans `hf-telematics-gw` qui produit des `RawPosition` ([[position-ingestion-pipeline]]). Poll ou push selon ce qu'ils offrent ; on préfère le push. Ce que l'adaptateur doit résoudre, dans l'ordre où ça a mordu :

- horodatage : heure du serveur ou de l'appareil, fuseau, format (voir [[incident-2026-02-trakko-timestamp-drift]]) ;

- identifiant de véhicule : stable ? lié à la plaque ? comment on l'obtient ([[tracker-vehicle-mapping]]) ;

- précision : fournie, ou à estimer ;

- pagination, curseurs, fenêtres glissantes, limites de débit ;

- signature des webhooks, et rejeu ;

- ce qu'on ne prend pas : vitesse (lue pour la plausibilité, pas stockée), conducteur, carburant, température.

**5. Jeu de fixtures dans le harnais.** Au moins : une heure normale de 5 véhicules, un rejeu, une rafale, des horodatages aberrants, un véhicule inconnu, un lot mal signé. Voir [[telematics-integration-test-harness]]. Le test de charge tourne avec le nouveau fournisseur ajouté au mélange.

**6. Écran de connexion transporteur.** `/settings/telematics` : comment un transporteur relie son compte (jeton, OAuth, identifiant de flotte), texte d'information au chauffeur qui nomme le fournisseur (exigence du projet conformité), création des mappings.

**7. Shadow run.** Deux semaines en production, derrière le drapeau `telematics.provider.<name>`, avec 20 véhicules d'un ou deux transporteurs volontaires. Les positions passent dans le pipeline et sont écrites, mais **ne sont pas la source affichée** ([[position-source-priority]] les met au rang le plus bas). On compare chaque jour : latence, taux de doublons, taux hors fenêtre, écart avec l'autre source quand il y en a une, arrivées détectées. Critères de sortie : p95 de latence sous l'objectif de sa catégorie, moins de 1 % de positions rejetées en plausibilité, zéro mapping faux sur les 20 véhicules.

**8. Ouverture.** Signature dans le ticket par la rota conformité (les points de l'étape 1 sont dans le contrat signé) et par le responsable produit dispatch (les critères de l'étape 7 sont atteints). Drapeau ouvert, priorité de source réglée, entrée ajoutée dans la liste publique des sous-traitants, note de coût mise à jour.

## Ce qu'on refuse à l'entrée

- Un fournisseur qui n'offre que du polling avec une limite de débit qui ne permet pas 30 s par flotte pour notre volume prévu.

- Un fournisseur qui exige d'installer son SDK dans notre application mobile.

- Un fournisseur qui ne peut pas nous donner l'heure serveur de réception.

## Durée constatée

Geolyx : 7 semaines (2025), dont 3 sur l'étape 1 parce que leur DPA standard prévoyait un traitement hors UE par leur support, corrigé par avenant. Le prochain fournisseur bénéficiera de cette note ; l'objectif est 5 semaines.
