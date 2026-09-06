---
name: forms-react-hook-form-zod
description: Forms use react-hook-form with zod schemas shared with the API error paths, server 422 errors are mapped onto fields by path, the load creation form autosaves a draft to sessionStorage
type: reference
status: active
verified: 2026-02-24
---

# Formulaires : react-hook-form + zod

## Schéma d'abord

Chaque formulaire a un schéma zod dans `src/forms/schemas/`, et le type TypeScript du formulaire est `z.infer<typeof loadFormSchema>`. Le schéma est la seule source de vérité de la validation côté client. Il est volontairement **moins strict** que le serveur : il vérifie la présence, les formats et les bornes évidentes (date d'enlèvement dans le futur, poids entre 1 et 40 000 kg), pas les règles métier qui dépendent de données serveur (le transporteur a-t-il une assurance valide).

Les messages de validation passent par `i18n` avec des clés `form.error.<code>`, jamais de texte en dur dans le schéma. `zodErrorMap` dans `src/forms/errorMap.ts` fait la traduction à partir du `code` de l'erreur zod.

## Erreurs serveur (422)

L'API renvoie `errors[]` avec `path` (chemin JSON, `pickup.window.start`) et `code`. `applyServerErrors(form, envelope)` fait `form.setError(path, { type: 'server', message: t('form.error.' + code) })` pour chaque entrée. Le `path` du serveur correspond à la structure du formulaire parce que le DTO envoyé **est** la valeur du formulaire, sans remappage. Quand un champ est renommé, il l'est des deux côtés dans la même PR, c'est le contrat.

Une erreur serveur dont le `path` ne correspond à aucun champ va dans `form.setError('root.server', ...)` et s'affiche en haut du formulaire. Ça arrive avec les règles métier transverses (`code: carrier_insurance_expired`).

## Brouillon

Le formulaire de création de chargement (14 champs, 3 étapes) sauvegarde ses valeurs dans `sessionStorage` sous `hf.draft.load` toutes les 2 secondes quand elles changent (`useDraftAutosave`). Au retour sur la page, une bannière propose de reprendre. Le brouillon est effacé à la soumission réussie. `sessionStorage` et pas `localStorage` : un brouillon qui survit à la fermeture du navigateur a produit des chargements en double le mois où on l'a essayé.

## Composants

Tous les champs passent par `src/forms/fields/` (`TextField`, `DateTimeField`, `AddressField`, `MoneyField`) qui prennent `name` et lisent le contexte du formulaire via `useController`. Un champ affiche son erreur sous lui, avec `aria-describedby` et `aria-invalid`, voir [[a11y-keyboard-drag-drop-dispatch]] pour les règles d'accessibilité générales.

`MoneyField` manipule des centimes entiers en interne et affiche avec `Intl.NumberFormat`. La valeur soumise est `{ amount: 125000, currency: 'EUR' }`. Jamais de nombre flottant pour un montant, ni dans le formulaire ni dans l'API.

`DateTimeField` prend une date et une heure locales et un fuseau (celui de l'adresse choisie), et produit un instant ISO. Voir [[date-time-display-rule]].

## Ce qu'on ne fait pas

- Pas de validation `onChange` sur les gros formulaires, `onBlur` puis `onSubmit`. Le `onChange` faisait clignoter les erreurs pendant la saisie.

- Pas de formulaires générés depuis le schéma OpenAPI. Testé, les libellés et l'ordre des champs sont un travail de produit, pas de schéma.
