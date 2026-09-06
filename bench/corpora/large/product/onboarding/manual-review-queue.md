---
name: manual-review-queue
description: The manual review queue: sorted by reviewer language then age, three reviewers, 8 business hour SLA, four verdicts with evidence
type: reference
status: active
verified: 2026-06-18
---

Tout ce qui ne passe pas automatiquement dans [[carrier-verification-steps]] arrive dans la file `/backoffice/onboarding/reviews`. Trois réviseurs (deux à Lyon, un à Varsovie pour le polonais et le roumain) la traitent de 08:00 à 18:00 CET en semaine.

## Ordre de traitement

Tri par langue du réviseur connecté d'abord (un réviseur voit en tête les dossiers dans les langues qu'il lit), puis par ancienneté du dossier. Un dossier qui attend plus de 8 heures ouvrées passe en rouge et remonte pour tous les réviseurs quelle que soit la langue. Objectif de délai : 8 heures ouvrées, tenu à 91 % en mai 2026, délai médian 4 heures.

Les dossiers `needs_call` (trois rejets automatiques, voir [[document-check-rules]]) sont dans un onglet séparé avec le numéro de téléphone du transporteur et les heures d'appel qu'il a indiquées.

## Verdicts

| Verdict | Effet | Preuve exigée |
|---|---|---|
| `manual_passed` | l'étape passe | le réviseur coche les points vérifiés (nom, numéro, date, montant) ; la liste dépend du type |
| `request_more` | email au transporteur avec un texte choisi parmi 14 motifs, le dossier sort de la file jusqu'au nouvel envoi | motif obligatoire |
| `rejected` | l'étape est rejetée, le transporteur peut renvoyer un document | motif obligatoire, texte libre affiché au transporteur |
| `fraud_suspected` | compte gelé, dossier transféré à la revue fraude, transporteur non notifié | note interne obligatoire |

Le verdict et la checklist sont stockés dans `carrier_verifications.evidence` avec `checked_by`. Un audit mensuel tire 100 dossiers `manual_passed` au hasard pour un second regard ; taux de désaccord en 2026 : 1,3 %, presque toujours sur un montant d'assurance en devise.

## Ce que le réviseur voit

Le document, l'OCR à côté avec les champs extraits et leur confiance, le résultat des vérifications automatiques (VIES, registre, voir [[vies-vat-check-integration]]), l'historique des envois du transporteur, et pour les licences le lien vers le registre national quand il existe ([[licence-community-check]]). L'objectif est que le réviseur ne quitte jamais l'écran.

## Volumes

Mai 2026 : 2 340 dossiers, 61 % de licences, 28 % d'assurances, 7 % d'extraits de registre allemands, 4 % d'identités en `identity_name_mismatch`. 74 % validés, 17 % `request_more`, 8 % rejetés, 1 % fraude suspectée.

## Ce qui a été refusé

Une validation « en un clic » sans checklist, demandée pour aller plus vite. Refusée : la checklist est ce qui rend l'audit possible et ce qui a permis de trouver les deux faux positifs sur les montants en zloty.
