---
name: document-check-rules
description: Accepted document formats (PDF, JPEG, PNG, HEIC converted, 15 MB max), the automatic rejections (screenshots, expired, cropped, mismatched company name), and the 3 attempt limit before a human call
type: reference
status: active
verified: 2026-04-09
---

Règles appliquées à tout document envoyé par un transporteur (`carrier_documents`), quel que soit le type (`licence`, `insurance_certificate`, `adr_certificate`, `identity`, `bank_proof`, `kbis_or_equivalent`).

## Formats

- PDF, JPEG, PNG. HEIC accepté à l'envoi depuis l'app mobile (les iPhone en produisent par défaut) et converti en JPEG côté serveur par `DocumentNormalizer`. Avant la conversion, 9 % des envois mobiles échouaient sur HEIC.
- 15 Mo maximum par fichier, 5 fichiers par document (une assurance peut faire plusieurs pages scannées séparément).
- Résolution minimale : 800 pixels sur le petit côté. En dessous, refus immédiat `document.resolution_too_low` avec un conseil (« prenez la photo plus près, en pleine lumière »).

## Rejets automatiques

`DocumentPreChecker` tourne à l'envoi, avant tout examen humain, et rejette avec un code précis :

- `document.screenshot` : l'image a les dimensions exactes d'un écran de téléphone connu et une barre d'état en haut. Les captures d'écran d'un document sont refusées parce qu'on ne peut pas vérifier qu'elles ne sont pas retouchées ; 6 % des envois.
- `document.expired` : la date d'expiration lue (OCR) est passée. Le transporteur peut contester si l'OCR a mal lu, ce qui arrive sur 2 % des lectures.
- `document.cropped` : moins de 3 des 4 coins du document sont visibles. Un document coupé cache souvent le nom de l'assuré ou la date.
- `document.company_mismatch` : le nom de société lu ne correspond pas à celui du compte (distance de Levenshtein normalisée au-dessus de 0,35 après suppression des formes juridiques). Le seuil a été monté de 0,25 à 0,35 en février 2026 parce que « Transports Duhamel et Fils SARL » contre « DUHAMEL TRANSPORTS » était rejeté.
- `document.wrong_type` : une licence envoyée dans la case assurance, détecté par classification d'image (précision 96 % sur l'échantillon de validation d'avril 2026).

## Limite de tentatives

Trois envois rejetés automatiquement pour le même type de document mettent le compte en `needs_call` : un membre de l'onboarding appelle le transporteur. On a mesuré que le quatrième envoi sans aide réussissait dans 18 % des cas seulement, alors qu'après un appel de 5 minutes on est à 80 %. Le coût de l'appel est largement couvert.

## Conservation

Les documents sont conservés tant que le compte est actif, plus 5 ans (obligation liée aux contrats de transport), dans le bucket `carrier-documents` avec chiffrement côté serveur. Une demande d'effacement d'un compte fermé depuis plus de 5 ans supprime les fichiers et garde une ligne `carrier_documents` avec `purged_at`.

Ce que les vérificateurs regardent ensuite est décrit dans [[manual-review-queue]] ; les règles spécifiques par type sont dans [[licence-community-check]] et [[insurance-certificate-parsing]].
