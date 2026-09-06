---
name: i18n-fr-en-strings
description: Dispatch front is translated with i18next in fr, en, de, pl and nl, keys are dotted and namespaced by screen, ICU plurals, missing keys fail CI, translators work in the JSON files through merge requests
type: reference
status: active
verified: 2026-05-08
---

# Internationalisation du front

## Outils

`i18next` + `react-i18next`, fichiers `src/locales/<lang>/<namespace>.json`. Langues : `fr` (source), `en`, `de`, `pl`, `nl`. Le `de` et le `nl` ont été ajoutés en 2026 pour les entités HF-DE et HF-NL. La langue vient du profil utilisateur (champ `locale`), avec repli sur `navigator.language` avant la connexion.

## Clés

- Format `namespace:screen.element.detail`, par exemple `board:card.status.bidding`, `forms:load.pickup.window_start.label`. Un namespace par grande zone (`common`, `board`, `forms`, `reports`, `messages`, `errors`).

- Jamais de texte en dur dans un composant. ESLint `i18next/no-literal-string` avec une liste d'exceptions courte (unités, symboles).

- Les clés d'erreur API sont `errors:<code>` où `<code>` est le slug du `type` de l'enveloppe d'erreur (`errors:load-not-biddable`). Un code inconnu affiche `errors:generic` et un événement `i18n.missing_error_key` part vers le monitoring pour qu'on l'ajoute.

## Pluriels et formats

ICU via `i18next-icu` : `"{count, plural, =0 {Aucune offre} one {# offre} other {# offres}}"`. Le polonais a quatre formes, ce qui est la raison du passage à ICU (les suffixes `_one` / `_other` d'i18next ne suffisaient pas pour `pl`).

Dates, nombres et montants : `Intl.DateTimeFormat`, `Intl.NumberFormat` avec la locale de l'utilisateur, jamais de formatage manuel. Les montants s'affichent avec la devise du chargement, pas celle de l'utilisateur. Voir [[date-time-display-rule]] pour les dates.

## Vérifications en CI

- `i18n:check` compare toutes les langues au `fr` : clé manquante = échec. Une clé peut être marquée `"__pending__"` dans une langue pendant au plus deux sprints, un compteur de `__pending__` par langue est affiché dans le résumé de la PR.

- Clé présente dans un JSON mais jamais utilisée dans `src/` = avertissement, puis suppression au nettoyage trimestriel.

- Placeholders : `{name}` présent dans le `fr` doit l'être dans chaque traduction, sinon échec.

## Processus de traduction

Pas d'outil de traduction externe. Les traductions `en` sont faites par l'équipe. Le `de`, `pl` et `nl` sont faits par des collègues des bureaux locaux directement dans les fichiers JSON via une merge request, avec un guide d'une page. Une clé nouvelle part en `fr` et `en` dans la même PR que la fonctionnalité, les autres langues suivent dans la semaine.

## Longueur des textes

Le `de` est en moyenne 30 % plus long que le `fr`, et ça casse les boutons à largeur fixe. Règle : aucune largeur fixe sur un élément qui contient du texte traduit, `min-width` seulement. Le playground de composants a un mode "pseudo-locale" qui allonge chaque chaîne de 40 % avec des caractères accentués pour tester avant la traduction.

## Chargement

Les fichiers sont importés statiquement pour la langue de l'utilisateur, avec `import()` dynamique par namespace non critique (`reports` et `messages`), ce qui s'ajoute au découpage décrit dans [[build-vite-chunking]].
