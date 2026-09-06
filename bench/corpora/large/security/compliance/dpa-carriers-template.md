---
name: dpa-carriers-template
description: Carriers sign DPA v3 where we are processor for driver data they upload and controller for accounts and app-collected positions; 4 negotiated variants
type: reference
status: active
verified: 2026-05-14
---

# Accord de traitement avec les transporteurs

## La question de départ

Pour un chauffeur, qui est responsable de traitement ? Le transporteur l'emploie et nous fournit son nom, son numéro, sa licence. Nous décidons comment les positions sont collectées, combien de temps, et nous les montrons au chargeur. La réponse retenue avec le conseil juridique, écrite dans l'annexe :

- **Le transporteur est responsable** des données de ses chauffeurs qu'il saisit ou téléverse (identité, documents, affectations). Nous sommes **sous-traitant** pour ces données : nous les traitons selon ses instructions, qui sont les conditions du service.

- **Nous sommes responsable** des données de compte (identifiants, connexions, journaux de sécurité), des positions collectées par notre application mobile (parce que nous en fixons les paramètres, voir [[driver-position-legal-basis]]) et des données de facturation.

- Le chargeur qui voit la position d'un camion pendant sa livraison est **destinataire**, pas responsable conjoint. Il ne reçoit pas le nom du chauffeur, seulement l'immatriculation et le prénom si le transporteur l'a activé.

Cette répartition mixte a été préférée à un « responsable conjoint » global, plus simple à écrire mais impossible à expliquer à un transporteur de trois camions.

## Le document

`DPA-carrier-v3` (février 2026), annexe aux conditions générales transporteur, acceptée par clic à la création du compte et re-présentée à chaque nouvelle version majeure. Le texte est dans le dépôt `halden-legal`, rendu en PDF avec un hash affiché en pied de page, et `organizations.dpa_version` + `dpa_accepted_at` + `dpa_accepted_by` enregistrent l'acceptation.

Sections :

1. Rôles (ci-dessus).

2. Objet et durée : la durée du contrat plus les rétentions de [[data-retention-matrix]], annexée.

3. Sous-traitants ultérieurs : liste publiée à `https://docs.halden.example/legal/subprocessors`, préavis de 30 jours par e-mail avant ajout, droit d'objection qui vaut résiliation sans frais. La liste inclut les fournisseurs télématiques, voir [[dpa-telematics-subprocessors]].

4. Mesures de sécurité : renvoi au document de mesures techniques et organisationnelles (le même que celui utilisé pour [[vendor-security-questionnaire]]).

5. Assistance : DSAR ([[dsar-handling-runbook]]) et notification de violation ([[breach-notification-72h-procedure]]) sous 48 h au transporteur.

6. Fin de contrat : export des données du transporteur sous 30 jours, puis suppression selon la matrice.

## Variantes négociées

Les gros transporteurs (plus de 200 camions) ont des services juridiques qui demandent des modifications. On accepte quatre types de variantes et on refuse le reste :

- Délai de notification de violation ramené à 24 h (accepté 3 fois).

- Audit sur site une fois par an à leurs frais, avec 30 jours de préavis (accepté 2 fois, jamais exercé).

- Hébergement dans l'Union européenne garanti contractuellement (c'est déjà le cas, on l'écrit).

- Liste nominative des sous-traitants au lieu du lien (accepté 1 fois, on leur envoie la liste à chaque changement).

Refusé systématiquement : la responsabilité illimitée, le droit de refuser un sous-traitant sans résilier, et une rétention des positions supérieure à la matrice « pour leurs propres besoins » (ils peuvent les exporter, pas nous les faire garder).

Chaque variante est une ligne de `dpa_variants (organization_id, clause, text, signed_at, signed_pdf_key)`. Quatre transporteurs en mai 2026.

## Ce qu'on a appris

La version v1 (2024) disait « responsable conjoint » partout. Deux transporteurs ont refusé de signer parce que leur assureur ne couvrait pas la responsabilité conjointe. La v2 a introduit la répartition, la v3 a ajouté les positions collectées par notre application comme traitement dont nous sommes responsables, après la discussion résumée dans [[dpo-feedback-position-data]].
