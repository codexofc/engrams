---
name: support-weekly-triage-ritual
description: Le tri hebdomadaire du lundi 14 h (support, un backend, un produit) qui relit les catégories de la semaine, décide ce qui devient ticket HF, et tient la liste des dix irritants
type: project
status: active
verified: 2026-06-22
---

# Tri hebdomadaire du lundi

Mis en place en HF-3008 (décembre 2025) parce que les mêmes tickets revenaient sans que personne ne les compte. Une heure, le lundi à 14 h, en visio, avec le support au complet, un backend (tournant, celui qui sort d'astreinte) et une personne du produit.

## Déroulé

1. **Les chiffres** (10 min). Le tableau Grafana « Support weekly » : volume par catégorie ([[support-ticket-tagging-taxonomy]]), respect des SLA ([[support-sla-tiers]]), tickets escaladés, tickets réouverts. On regarde les écarts avec la semaine précédente, pas les valeurs absolues.

2. **Les `other`** (10 min). Chaque ticket `other` de la semaine est lu à haute voix en une phrase. Soit on lui trouve une catégorie, soit on note qu'une catégorie manque. Trois semaines de suite avec le même manque et on ajoute la catégorie.

3. **Les escaladés** (20 min). Pour chaque ticket passé en L2 ou L3 : le playbook a-t-il suffi ? S'il n'existait pas, qui l'écrit cette semaine ? S'il existait mais était faux, qui le corrige ? Un playbook faux est pire qu'absent.

4. **Les dix irritants** (15 min). Une liste tenue dans le wiki support, ordonnée par volume de tickets sur quatre semaines glissantes. Chaque ligne a un ticket HF ou la mention « accepté, pas de correctif prévu » avec la raison. Le produit tient la liste à jour avec la roadmap.

5. **Décisions** (5 min). Ce qui change dans les macros, les playbooks, les catégories. Écrit dans le compte rendu, un message dans `#support`.

## Les dix irritants en juin 2026

Pour mémoire, dans l'ordre :

1. Chauffeur qui a oublié son PIN et dont le transporteur ne sait pas envoyer le lien (HF-3155, bouton dans le back-office transporteur, livré).

2. Chargement `DISPATCHED` sans pickup parce que le chauffeur n'a pas l'app installée (HF-3160, SMS de rappel J-1, en cours).

3. Intégrateur qui ne reçoit pas ses webhooks parce que son abonnement a été désactivé pour trop d'échecs et que l'e-mail est parti à un ancien admin (HF-3121, livré).

4. Facture en PLN dont le client attend l'arrondi de sa propre compta (accepté, pas de correctif : on suit la règle d'arrondi de la facturation).

5. Chargement Fretzone en double parce que le chargeur l'a aussi publié en direct (HF-3210, en cours).

6. Recherche transporteur qui ne trouve pas un chargement publié il y a moins de 30 s (accepté : le délai d'indexation est documenté).

7. Photo de POD floue refusée par le chargeur (accepté : demande produit, pas un bug).

8. Note de transporteur qui baisse après un retrait légitime (HF-3170, revue de la pénalité).

9. E-mail de facture dans les spams chez deux grands hébergeurs (ops, DMARC, livré).

10. Compte chargeur SSO qui ne retrouve plus son accès après changement de fournisseur d'identité (HF-3180).

## Ce qu'on a appris

Le tri est plus utile pour ce qu'il retire de la liste que pour ce qu'il y met. « Accepté, pas de correctif » est une vraie décision : L1 sait quoi répondre, la macro existe, et on arrête de réescalader.

Le backend tournant est ce qui fait tenir le rituel. Quand c'est toujours la même personne, elle finit par tout prendre sur elle et le reste de l'équipe backend ne sait plus ce que le support vit.
