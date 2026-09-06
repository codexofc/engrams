---
name: on-call-rotation-platform
description: Platform on-call is one backend and one front-or-mobile engineer per week, paged only by the alerts listed as paging, with a 15 min acknowledge target, a runbook link in every alert, and a post-incident note within 3 working days
type: project
status: active
verified: 2026-06-09
---

# Astreinte plateforme

Mise en place en octobre 2025 (HF-1180) en remplacement du "celui qui voit l'alerte en premier". L'astreinte ops (cluster, base, réseau) est séparée, voir les notes du projet Kubernetes.

## Rotation

- Une semaine, du lundi 9 h au lundi 9 h. Deux personnes : une backend, une front ou mobile. 6 personnes éligibles côté backend, 5 côté front et mobile, donc une semaine sur 6 ou 5.

- Le planning est dans le calendrier partagé six semaines à l'avance. Un échange se fait entre les deux personnes concernées et se note dans le calendrier, pas dans un message.

- Pas d'astreinte la semaine qui suit une semaine d'astreinte. Pas d'astreinte pour quelqu'un qui a moins de 3 mois d'ancienneté.

## Ce qui page

Seules les alertes avec le label `severity: page` réveillent quelqu'un. La liste est courte et se révise à chaque rétro :

- `ApiErrorRateHigh` (5xx > 2 % pendant 5 min)

- `ApiLatencyP99High` (> 3 s pendant 10 min)

- `MessengerFailedQueueGrowing` (> 100 messages en 15 min)

- `OutboxRelayDown` (relais webhook sans activité pendant 5 min alors qu'il y a des lignes en attente)

- `MobileSyncErrorRateHigh` (> 10 % de 5xx sur `/internal/mobile/sync` pendant 5 min)

- `WebBootErrorSpike` (> 20 erreurs de démarrage par minute, ajouté après l'incident Safari)

- `InvoicingRunFailed` (le job de facturation mensuel a échoué, page même la nuit)

Tout le reste (`severity: warn`) va dans le canal d'alertes et se regarde aux heures de bureau.

## Engagement

- Accusé de réception en 15 minutes, jour et nuit. Première communication dans le canal d'incident en 30 minutes, même si c'est "je regarde".

- Chaque alerte de page a un lien `runbook` dans ses annotations. Une alerte sans runbook ne passe pas en `page`, c'est vérifié par un test sur les règles.

- La personne d'astreinte peut tout faire pour rétablir le service : rollback, désactivation de flag, blocage d'une version mobile à l'ingress, montée de `min-version`. Elle prévient, elle ne demande pas la permission.

- Elle ne fait pas de correctif de code la nuit. Rollback ou contournement, et le correctif attend le matin.

## Après

- Une note d'incident dans les 3 jours ouvrés, dans la mémoire du projet concerné, avec chronologie, cause, correctifs, et ce qui n'a pas été fait. Les incidents documentés cette année suivent tous ce format.

- Une revue d'incident de 30 minutes la semaine suivante, sans recherche de responsable, dont sortent des tickets.

- Compensation : une demi-journée de récupération par nuit avec un réveil, prise dans les deux semaines.

## Chiffres depuis octobre 2025

23 pages en 8 mois, dont 6 de nuit. 4 fausses alertes (toutes corrigées par un ajustement de seuil). Temps médian d'accusé de réception : 6 minutes. Temps médian de rétablissement : 35 minutes. L'incident le plus long est celui de la migration de novembre 2025 (documenté côté API), qui a précédé de peu la mise en place de l'astreinte et l'a justifiée.
