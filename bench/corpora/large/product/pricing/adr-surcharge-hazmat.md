---
name: adr-surcharge-hazmat
description: ADR surcharge by class (15, 25 or 35 %), limited quantities exempt, and a valid ADR certificate required to bid
type: reference
status: active
verified: 2026-03-19
---

Un chargement marqué ADR (`loads.adr_class` non nul, une des classes 1 à 9 avec sous-divisions pour la classe 1) déclenche la règle 4 du [[surcharge-rules-catalog]] et une contrainte sur les transporteurs autorisés à enchérir.

## Taux par classe

| Classe | Exemples | Majoration |
|---|---|---|
| 3, 8, 9 | liquides inflammables, corrosifs, divers (batteries lithium en 9) | +15 % |
| 2, 4, 5, 6 | gaz, solides inflammables, comburants, toxiques | +25 % |
| 1, 7 | explosifs, radioactifs | +35 % |

Table `adr_classes` (`class`, `surcharge_pct`, `requires_specialist`, `label_fr`, `label_de`, `label_pl`). Les classes 1 et 7 ont `requires_specialist = true` : seuls les transporteurs avec le drapeau `carrier_accounts.adr_specialist` (validé à la main par l'onboarding) voient ces chargements.

Les quantités limitées (LQ) et les quantités exceptées (EQ) ne sont pas ADR au sens de la majoration : `loads.adr_limited_quantity = true` annule la majoration mais garde l'affichage du pictogramme, parce que le conducteur doit quand même savoir ce qu'il transporte. Cette distinction a été ajoutée en janvier 2026 (HF-2410) après qu'un chargeur de peintures a payé +15 % sur 200 chargements en LQ.

## Contrainte sur les transporteurs

Enchérir sur un chargement ADR exige un certificat ADR conducteur en cours de validité dans les documents du transporteur (`carrier_documents.type = 'adr_certificate'`, `expires_at > pickup_at`). Le contrôle est fait par `bid-svc` à l'ouverture du formulaire : sans certificat, le formulaire est remplacé par le message `bid.adr_certificate_required` et un lien vers l'envoi de document. Les règles de vérification des documents sont chez l'onboarding.

Un certificat qui expire entre l'enchère et le chargement : le chargement reste attribué mais le dispatch reçoit une alerte 48 h avant. Le cas est rare (14 fois en 2025) et on n'annule pas automatiquement.

## Ce que la majoration ne couvre pas

- Le matériel spécifique (citerne, conteneur-citerne) est un type de véhicule, pas une majoration ; il change le cluster de ligne ([[pricing-lane-clusters]]).
- Le conseiller à la sécurité que le chargeur doit avoir est sa responsabilité, on ne vérifie rien de ce côté.
- Les tunnels interdits (catégories B à E) sont pris en compte dans l'itinéraire, donc dans `toll` et dans la distance, pas dans `adr`. Voir [[toll-cost-tables]].

## Volumes

ADR représente 3,1 % des chargements, dont 78 % en classe 3, 8 ou 9. Classe 1 : 31 chargements en 2025, tous par quatre transporteurs.
