---
name: support-escalation-path
description: Support escalates in three tiers (agents, on-duty product engineer, on-call), labels and targets, four cases that skip to on-call
type: reference
status: active
verified: 2026-05-15
---

## Niveaux

**Niveau 1, support.** Six agents (Lyon et Varsovie), 07:00 à 20:00 CET en semaine, 09:00 à 17:00 le samedi. Ils traitent les demandes des chargeurs, transporteurs et conducteurs avec le back-office et les macros. Objectif de première réponse : 2 heures en semaine.

**Niveau 2, ingénieur produit de permanence par domaine.** Chaque domaine (billing, pricing, onboarding, dispatch, app conducteur) a une personne de permanence par semaine, indiquée dans le calendrier `product-duty`. Le support escalade en créant un ticket `HF-` avec le label `support-escalation` et le label du domaine, et en le mentionnant dans le canal du domaine. Objectif de prise en charge : 4 heures ouvrées. Le ticket porte le numéro du compte, les identifiants concernés (`load_id`, `invoice_id`) et ce que le support a déjà vérifié.

**Niveau 3, astreinte.** L'astreinte technique (24/7) est déclenchée par l'ingénieur de permanence, pas par le support, sauf dans les quatre cas ci-dessous.

## Ce qui va directement à l'astreinte

1. Un incident de paiement de masse : plusieurs chargeurs signalent un prélèvement en double ou un montant erroné dans la même heure.
2. Une fuite de données : un utilisateur voit les données d'un autre compte.
3. L'app conducteur inutilisable (impossible de charger ou de signer un POD) signalé par plus de trois transporteurs.
4. Un blocage de facturation le jour de clôture (le 1er et les jours ouvrés 1 à 3).

Pour ces quatre cas, le support appelle le numéro d'astreinte directement, puis crée le ticket.

## Ce que le niveau 2 doit faire

- Répondre dans le ticket, pas seulement dans le canal, pour que le support ait une trace à donner au client.
- Si c'est un bug, créer le ticket de correction lié par `Depend` au ticket d'escalade et dire au support la date probable, ou « pas de date » si c'est le cas. Le support préfère « pas de date » à un silence.
- Si c'est un comportement voulu, l'expliquer en une phrase que le support peut réutiliser, et proposer une macro si la question est revenue trois fois.

## Chiffres

Avril 2026 : 3 900 conversations support, 210 escalades niveau 2 (5,4 %), 4 déclenchements d'astreinte par le support (dont 2 justifiés). Domaines des escalades : billing 38 %, onboarding 27 %, dispatch 18 %, pricing 9 %, app conducteur 8 %.

Le support a accès en lecture aux notes de version internes ([[release-notes-rules]]) et à l'état des flags, ce qui a réduit les escalades « est-ce que c'est normal que cet écran ait changé ».
