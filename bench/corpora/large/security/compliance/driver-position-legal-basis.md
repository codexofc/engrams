---
name: driver-position-legal-basis
description: Driver tracking rests on legitimate interest (BT-03), limited to active assignments plus 30 min, with an objection path that switches to manual statuses
type: project
status: active
verified: 2026-02-27
---

# Fondement juridique du suivi de position des chauffeurs

Cette note est le résumé opérationnel du test de mise en balance `BT-03` (voir [[records-of-processing-register]]), rédigé pour l'équipe et pas pour le juriste. Le contexte de la discussion est dans [[dpo-feedback-position-data]].

## La finalité, précisément

Deux finalités, et pas une de plus :

1. **Montrer au chargeur où est son chargement** pendant l'exécution, et lui donner une heure d'arrivée estimée. C'est le service vendu. Un chargeur qui ne voit pas le camion appelle le dispatcher, qui appelle le chauffeur, qui répond au téléphone en conduisant.

2. **Constater l'exécution** : heure d'arrivée réelle à l'enlèvement et à la livraison, pour la facturation des attentes et pour les litiges.

Ce qui n'est pas une finalité : évaluer la conduite du chauffeur, mesurer ses pauses, optimiser ses tournées à son insu, alimenter un score individuel. Les résumés de trajets (`trip_summaries`) ne contiennent pas de vitesse et les arrêts y sont arrondis à 15 minutes et à la commune, précisément pour que ces usages soient impossibles avec nos données.

## Pourquoi l'intérêt légitime

Trois fondements ont été examinés :

- **Consentement** : écarté. Un chauffeur salarié n'est pas libre de refuser l'outil de son employeur. Un consentement non libre est nul, et on aurait construit tout le service sur du sable.

- **Exécution du contrat** : le contrat est entre nous et le transporteur (ou nous et le chargeur), pas avec le chauffeur. Applicable seulement aux chauffeurs indépendants qui sont leur propre transporteur, environ 12 % des comptes chauffeur. Retenu pour eux à titre subsidiaire.

- **Intérêt légitime** : retenu. Notre intérêt (fournir le service de suivi qui est la raison d'être de la plateforme) et celui du chargeur (savoir où est sa marchandise) sont réels ; le traitement est nécessaire (on a essayé les mises à jour manuelles de statut en 2023, taux de mise à jour 40 %, ETA inutilisables) ; et l'impact sur le chauffeur est limité par les garanties ci-dessous.

## Les garanties

C'est la partie qui fait tenir le test.

- **Fenêtre stricte** : collecte entre le début et la fin de la mission, plus 30 minutes avant le créneau d'enlèvement. Appliqué dans l'application mobile (HF-2093) et à l'ingestion pour les trackers matériels.

- **Rétention courte** : 90 jours de brut, 24 mois de résumés dégradés (voir [[data-retention-matrix]]).

- **Pas de vitesse, pas de score.** Les positions brutes ont une vitesse instantanée fournie par le GPS ; elle est écartée à l'ingestion, pas stockée.

- **Transparence** : statut visible dans l'application, information à la première connexion, mention du fournisseur télématique.

- **Destinataires limités** : le chargeur voit la position pendant sa mission, pas avant, pas après, sans le nom du chauffeur. Le transporteur voit ses véhicules. Personne d'autre, y compris chez nous (le support voit la dernière position, pas la trace, sauf par impersonation tracée).

- **Pas de décision automatisée** fondée sur la position : un retard détecté produit une notification au dispatcher, jamais une pénalité automatique.

## Le droit d'opposition

Un chauffeur peut s'opposer. Ce n'est pas un bouton (ce serait un consentement déguisé), c'est une demande par la voie DSAR ([[dsar-handling-runbook]]) ou par son employeur, examinée au cas par cas. Notre position par défaut, écrite dans BT-03 : on accepte, sauf si le transporteur fait valoir un motif impérieux (marchandise sous température dirigée avec obligation de traçabilité, par exemple).

Ce que ça donne concrètement : `users.tracking_opted_out = true`. L'application n'envoie plus de position ; le chauffeur passe en **mise à jour manuelle de statut** (boutons « Arrivé à l'enlèvement », « Parti », « Arrivé à la livraison »). Le chargeur voit les statuts, pas la carte, avec la mention « suivi manuel ». Le chauffeur garde ses missions. Le transporteur est informé. Trois oppositions en 2025, toutes acceptées ; deux chauffeurs sont revenus au suivi automatique de leur propre initiative parce que les appels du dispatcher avaient repris.

## Ce que le juriste a demandé qu'on écrive

- Que le tracker matériel installé par le transporteur suit le véhicule en continu par construction et que nous n'en sommes pas le décideur ; nous filtrons ce que nous recevons. La responsabilité du suivi continu est celle du transporteur, et le DPA le lui dit ([[dpa-carriers-template]]).

- Que le test est revu tous les 24 mois ou à chaque changement de finalité. Prochaine revue : novembre 2027, ou avant si quelqu'un propose d'utiliser les positions pour autre chose que les deux finalités ci-dessus, ce qui arrive environ une fois par trimestre et reçoit la même réponse.

## Résumé pour l'équipe produit

Quand quelqu'un propose une fonctionnalité qui touche aux positions, trois questions à se poser avant d'ouvrir un ticket :

1. Est-ce l'une des deux finalités (montrer où est le chargement, constater l'exécution) ? Si non, il faut un nouveau test de mise en balance, et le DPO dira probablement non.

2. Est-ce que ça marche avec 90 jours de brut et des résumés dégradés ? Si la réponse est « il faudrait garder plus longtemps », la réponse est non.

3. Est-ce que le chauffeur le verrait ? Une fonctionnalité de position que le chauffeur ne peut pas voir dans son application est une fonctionnalité de surveillance.

Trois oui, on avance. Un non, on en parle avec la rota conformité avant d'écrire une ligne.
