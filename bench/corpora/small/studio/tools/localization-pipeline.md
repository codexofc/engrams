---
name: localization-pipeline
description: Strings in strings/<lang>.toml with dotted keys, FR as source, 6 target languages, missing keys fail the playtest lane
type: project
status: active
verified: 2026-05-15
---

## Format

Un fichier par langue, `strings/fr.toml`, `strings/en.toml`, etc., clés à points (`ui.sail.raise`, `dialogue.harbourmaster.03`), valeurs avec des variables entre accolades (`{count}`) et des formes plurielles par un suffixe (`.one`, `.other`, `.many` pour le polonais). Le français est la langue source : une clé n'existe que si elle existe en français.

Le pipeline ([[asset-pipeline-overview]]) compile chaque fichier en une table binaire (`strings/<lang>.bin`) avec les clés hachées ; le moteur charge une table par langue au démarrage, 1,1 Mo pour le français en mai 2026 (9 400 clés).

## Langues

FR (source), EN, DE, ES, IT, PL, PT-BR. La traduction est faite par un prestataire à partir d'un export CSV hebdomadaire (`brumepipe strings export`) et réimportée par `brumepipe strings import <csv>`, qui refuse une clé absente du français et signale les valeurs identiques au français (souvent une traduction oubliée, parfois un nom propre).

## Vérifications

- **Merge request** : clés manquantes dans une langue cible signalées en avertissement (le prestataire a une semaine de retard par construction).
- **Lane playtest** : clés manquantes en erreur pour les langues du playtest (FR et EN). Une chaîne non traduite dans un build de playtest s'affiche comme `[ui.sail.raise]` et fait perdre du temps aux testeurs.
- Un test joue chaque écran d'UI dans chaque langue et mesure le débordement de texte ; l'allemand déborde sur 6 écrans en mai 2026, ticket BR-410 chez l'UI.

## Décisions

- Pas de chaînes dans le code, ni dans les fichiers de niveau. Le lint `no-literal-ui-string` cherche les littéraux passés aux fonctions d'UI.
- Les dialogues sont dans les mêmes fichiers que l'UI, pas dans un outil séparé ; l'équipe narrative écrit dans le TOML avec un aperçu dans l'éditeur. Un outil de dialogue a été discuté et remis après la sortie.
- Les nombres et dates sont formatés par langue par le moteur, jamais dans la chaîne.
