---
name: dpo-feedback-position-data
description: The DPO concluded continuous GPS of a named driver is surveillance unless tied to an active load; led to window filtering, 90 day raw and visible status
type: feedback
status: active
verified: 2026-01-30
---

# Ce que la revue du DPO nous a appris sur les positions

Revue menée en novembre et décembre 2025 par le DPO externe, à l'occasion de la refonte de la matrice de rétention ([[data-retention-matrix]]) et du test de mise en balance des positions (voir [[driver-position-legal-basis]]). Ce qui a été dit, ce qu'on a changé, ce qu'on a refusé de changer.

## Le constat

« Vous suivez des personnes, pas des camions. » L'application mobile collectait des positions dès que le chauffeur était connecté, chargement actif ou non, parce que c'était plus simple pour la détection d'arrivée au chargement suivant. Le DPO a fait remarquer que la position d'un chauffeur connecté un dimanche sans mission est la position d'une personne pendant son temps libre, et qu'aucune finalité du registre ne la couvre.

Deuxième point : la rétention de 12 mois en brut (matrice 2025) n'était justifiée par rien de mesuré. « Litiges » sans chiffre, c'est une intuition, pas un fondement.

Troisième point : le chauffeur ne savait pas quand il était suivi. L'icône de tracking de l'application était permanente et identique, mission ou pas.

## Ce qu'on a changé

1. **Collecte liée au chargement.** Depuis HF-2093 (janvier 2026), l'application ne collecte des positions qu'entre `assignment.started` et `assignment.completed` (plus une marge de 30 minutes avant le créneau d'enlèvement pour la détection d'approche). Hors de ces fenêtres, aucun point ne quitte le téléphone. Les trackers matériels (Trakko) suivent le véhicule en continu par construction ; pour eux, la règle est appliquée à l'ingestion : les points hors fenêtre de mission sont écartés avant écriture. Le projet d'intégration télématique détaille le filtre.

2. **90 jours de brut**, puis résumés de trajets. Décision détaillée dans la matrice. Les dispatchers ont perdu la vue fine au-delà de 90 jours ; personne ne s'en est plaint au bout de trois mois.

3. **Statut visible.** L'icône de suivi a trois états : gris (pas de collecte), vert (collecte pour la mission X), orange (collecte suspendue, réseau indisponible, points en attente). Un appui long explique. Retour des chauffeurs en test : « au moins on sait ».

4. **Information au chauffeur** à la première connexion et à chaque changement de fournisseur télématique, nommant le fournisseur et la durée.

## Ce qu'on a refusé, et pourquoi

- **Le consentement comme fondement.** Le DPO l'a envisagé, on l'a écarté ensemble : un chauffeur ne peut pas refuser librement un outil de travail, un consentement dans ce contexte ne vaut rien. L'intérêt légitime, avec des garanties fortes, est plus honnête. Le droit d'opposition existe et est traité au cas par cas (voir la note sur le fondement).

- **Anonymiser la position vue par le chargeur.** Proposé : montrer le camion sans identifiant. Refusé par le produit et accepté par le DPO : le chargeur voit déjà l'immatriculation sur le quai, la cacher sur la carte ne protège rien et casse la réconciliation à la livraison. Le nom du chauffeur, lui, n'est pas montré au chargeur, seulement le prénom si le transporteur l'active.

## Ce qu'on garde comme réflexe

- Une donnée de localisation sans fenêtre de finalité est une donnée de surveillance. La question à poser sur toute nouvelle collecte est « entre quel événement et quel événement ».

- Une durée de rétention se justifie par un chiffre (les litiges se sont tous ouverts sous 30 jours), pas par un adjectif.

- La personne suivie doit pouvoir voir qu'elle l'est, au moment où elle l'est.
