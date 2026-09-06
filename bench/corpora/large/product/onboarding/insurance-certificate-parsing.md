---
name: insurance-certificate-parsing
description: InsuranceExtractor reads certificates by OCR and 31 insurer templates, auto-pass needs cover of 100 000 EUR at confidence 0.8
type: project
status: active
verified: 2026-05-28
---

## Le problème

L'attestation d'assurance marchandises (responsabilité CMR) est le document le moins standardisé du parcours. Chaque assureur a sa mise en page, en cinq langues, et certaines attestations sont des lettres libres. C'est l'étape 4 de [[carrier-verification-steps]] et celle qui passe le moins souvent automatiquement.

## Extraction

`InsuranceExtractor` fait :

1. OCR de toutes les pages (après les pré-contrôles de [[document-check-rules]]).
2. Détection de l'assureur par une liste de 140 noms et logos connus. 82 % des attestations viennent de 12 assureurs.
3. Si un gabarit existe pour cet assureur (`insurance_templates`, 31 gabarits en mai 2026), extraction par zones : nom de l'assuré, numéro de police, montant de garantie CMR, date de fin de validité. Chaque champ a une confiance.
4. Sinon, extraction générique par expressions régulières multilingues (« montant garanti », « Deckungssumme », « suma gwarancyjna ») avec une confiance plafonnée à 0,7, donc jamais d'auto-validation sans gabarit.

## Règle d'auto-validation

Tous les champs lus avec une confiance d'au moins 0,8, et :

- nom de l'assuré correspondant au compte (même règle de distance que le reste),
- montant de garantie d'au moins 100 000 EUR (ou équivalent au taux du jour pour PLN, RON, CZK),
- date de fin après aujourd'hui plus 15 jours (une attestation qui expire dans une semaine n'est validée qu'à la main, pour que le réviseur demande déjà le renouvellement).

Le montant de garantie est aussi stocké dans `carrier_accounts.insurance_cover_cents` et sert au blocage des chargements de valeur déclarée supérieure : un transporteur assuré à 100 000 EUR ne peut pas être attribué un chargement déclaré à 150 000 EUR. Les chargeurs déclarent la valeur sur 35 % des chargements seulement ; sans déclaration, on prend 20 000 EUR par défaut.

## Résultats

Mai 2026 : 48 % d'auto-validation (contre 29 % en novembre 2025, avant les gabarits). Sur les 52 % restants, 70 % sont validés en revue manuelle sans échange avec le transporteur, 22 % demandent un document complémentaire (le plus souvent l'attestation ne mentionne pas le montant CMR, seulement la responsabilité civile), 8 % sont rejetés (attestation expirée non détectée par l'OCR, ou couverture inférieure à 100 000 EUR).

Faux positifs de l'auto-validation (validé automatiquement, puis contesté à l'audit mensuel de 100 dossiers) : 2 sur 300 dossiers audités depuis janvier, les deux sur un montant lu en PLN et pris pour des euros. Corrigé en avril : la devise est un champ extrait avec sa propre confiance, et un montant sans devise lisible ne passe plus automatiquement.

## Prochain gabarit

Les gabarits sont ajoutés par ordre de volume. Le prochain est un courtier roumain qui représente 6 % des attestations RO et dont les lettres changent de mise en page tous les trimestres ; on hésite à le faire.
