---
name: weekend-surcharge-backfire
description: Incident of 2025-09-13 where cascading percentage surcharges compounded a weekend ADR frigo quote to +71 %, 3 400 inflated quotes in a weekend, and the switch to surcharges applied on the base price only
type: project
status: active
verified: 2025-10-06
---

## Ce qui s'est passé

Le samedi 2025-09-13, un chargeur a signalé un devis de 2 940 EUR pour un Lille-Anvers frigo ADR classe 3, contre 1 700 EUR habituellement. En regardant la décomposition : base 1 720 EUR, `fuel` +5,1 %, `weekend` +12 %, `adr` +15 %, `temperature` +18 %. Attendu : 1 720 × 1,501 = 2 582 EUR. Obtenu : 2 940 EUR, soit +71 %.

Cause : la version de `surcharges/engine.py` déployée le 2025-09-11 (HF-2015, refactoring pour introduire l'ordre d'évaluation) appliquait chaque pourcentage sur le prix déjà majoré au lieu du prix de base. 1,051 × 1,12 × 1,15 × 1,18 = 1,597, et une erreur supplémentaire sur `weekend` qui comptait deux fois (chargement samedi et livraison dimanche, chaque jour ajoutant +12 %) donnait 1,71.

Sur un chargement sans majoration, rien ne changeait, ce qui explique que les tests et la revue soient passés : les cas de test n'avaient qu'une majoration à la fois.

## Impact

- 3 400 devis émis entre le vendredi 2025-09-12 18:00 et le lundi 2025-09-15 09:30 avec au moins deux majorations en pourcentage.
- 610 chargements attribués sur ces devis. Prix attribué médian à 1,14 de la suggestion corrigée, contre 1,04 en temps normal : les transporteurs ont suivi la suggestion gonflée.
- 41 réclamations de chargeurs. On a émis des avoirs de la différence sur les commissions (la commission est un pourcentage du prix attribué, donc elle aussi était gonflée), 6 200 EUR au total. Le prix transporteur, lui, est un contrat entre chargeur et transporteur, on n'y a pas touché.

## Correctifs (HF-2022)

- Toutes les majorations en pourcentage s'appliquent sur le prix de base. Somme des pourcentages, puis application une fois. C'est désormais la règle écrite dans [[surcharge-rules-catalog]].
- `weekend` est un booléen : un chargement ou une livraison en week-end, une seule fois.
- Les tests combinatoires : `test_all_pairs_of_surcharges` génère chaque paire de règles et vérifie que le total est la somme, et un test de référence sur 50 devis réels d'août 2025 rejoués avec `pricing-svc quote --replay` qui doit redonner le même montant au cent.
- Une alerte sur la distribution : si la médiane de `total_surcharge_pct` sur une heure dépasse de 5 points la médiane des 7 jours précédents, le canal pricing reçoit un message. Elle se serait déclenchée le vendredi à 19:00.

## Ce qu'on retient

Un refactoring de calcul de prix se teste sur des devis réels rejoués, pas sur des cas unitaires à une variable. Le rejeu est maintenant dans la CI de `pricing-svc` (200 devis, 4 secondes).
