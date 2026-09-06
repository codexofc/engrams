---
name: geolyx-contract-renewal-2026
description: Geolyx renewed in May 2026 for 24 months at 1.20 EUR per active vehicle, adding the replay header, a 99.5 % delivery SLA with credits and a 6 month exit
type: project
status: active
verified: 2026-05-27
---

# Renouvellement Geolyx 2026

Le premier accord partenaire (juin 2025, 12 mois) arrivait à échéance. Ticket HF-2158, négociation de mars à mai 2026, signature le 2026-05-20. Ce qu'on a demandé, obtenu, lâché.

## Point de départ

- 320 véhicules en novembre 2025, 330 en mai 2026 : la part Geolyx ne grandit pas beaucoup, parce que les flottes moyennes qui l'utilisent étaient déjà chez nous. La croissance est sur l'application.

- Coût 1,20 EUR par véhicule actif, gratuit pour le transporteur ([[telematics-costs-per-provider]]). Geolyx voulait passer à 1,60.

- Deux irritants techniques : le rejeu massif après panne ([[incident-2025-11-geolyx-duplicate-flood]]) et l'absence d'engagement de livraison des webhooks.

- Un point juridique : l'accord de 2025 renvoyait à leurs conditions générales pour la localisation du traitement, corrigées par avenant à l'époque ; on voulait le texte dans le contrat lui-même.

## Ce qu'on a obtenu

- **Prix inchangé** à 1,20 EUR par véhicule actif, 24 mois. Argument qui a porté : la gratuité côté transporteur est ce qui fait que leurs clients nous préfèrent à un boîtier, et un prix plus haut nous aurait poussés à facturer, donc à réduire leur usage. Ils l'ont compris comme on l'espérait.

- **En-tête `X-Geolyx-Replay: true`** sur tout lot rejoué après incident, contractualisé (déjà livré en janvier, mais un engagement écrit évite qu'une refonte de leur côté le fasse disparaître). Notre pipeline traite ces lots à basse priorité ([[position-ingestion-pipeline]]).

- **SLA de livraison** : 99,5 % des positions livrées à notre endpoint dans les 60 s de leur réception, mesuré mensuellement sur leur horodatage `sent_at` contre notre réception, avec un rapport qu'ils produisent et qu'on peut contester avec nos propres chiffres (`telematics_time_gap_seconds`). Crédit de 10 % de la facture du mois par tranche de 0,5 point sous l'objectif, plafonné à 50 %. Premier rapport : juin 2026.

- **Traitement dans l'Union européenne**, support inclus, dans le corps du contrat et non dans une annexe renvoyant à leurs conditions. Notification de violation sous 48 h.

- **Sortie** : préavis de 6 mois de part et d'autre, et à la fin, suppression de nos données de leur côté sous 30 jours avec **certificat de suppression** nominatif pour notre compte partenaire. Un transporteur qui quitte notre plateforme reste client Geolyx ; on ne peut pas exiger la suppression de ses données chez eux, on demande seulement la fin du flux vers nous, ce que le contrat dit clairement.

## Ce qu'on a lâché

- Une limite de débit de leur côté sur les rejeux (« lisser à 200 lots par minute »). Refusé une seconde fois. On a le header et notre file à basse priorité, ça suffit.

- L'accès à la température des remorques frigorifiques, que trois transporteurs demandaient. Geolyx le facture 0,40 EUR par véhicule en plus ; on n'a pas de fonctionnalité qui l'utilise et la conformité aurait demandé une finalité. Reporté, pas refusé.

- Un engagement de leur part sur la stabilité du format du webhook : ils promettent 6 mois de préavis pour un changement incompatible, pas plus. Notre adaptateur ([[geolyx-push-webhook-format]]) est de toute façon le seul endroit qui connaît le format.

## Ce que ça change chez nous

- `telematics_subscriptions` gagne `contract_version = '2026'` pour distinguer, dans les rapports de SLA, les abonnements créés sous l'ancien accord (aucune différence technique, mais le rapport SLA ne couvre que les nouveaux jusqu'à migration, faite en juin).

- Le rapport SLA mensuel est calculé de notre côté par `telematics:sla-report --provider geolyx --month 2026-06` et comparé au leur ; l'écart est le point de départ d'une contestation, pas leur chiffre seul.

- La liste des sous-traitants du projet conformité a été mise à jour avec la date du nouveau contrat.

Prochaine échéance : mai 2028. Rappel posé dans l'inventaire des contrats à 8 mois avant, parce que trois mois de négociation plus le préavis de 6 mois ne laissent pas de marge si on veut pouvoir partir.
