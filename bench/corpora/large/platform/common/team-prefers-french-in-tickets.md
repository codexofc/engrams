---
name: team-prefers-french-in-tickets
description: The platform team writes tickets and MR descriptions in French with English identifiers left as-is, code and commit subjects on the API and infra repos in English, and nobody translates error messages
type: user
status: active
verified: 2026-01-12
---

# Langue de travail de l'équipe plateforme

Décidé en rétro (novembre 2025) après six mois de flottement où chaque personne écrivait dans la langue qui lui venait, ce qui donnait des tickets bilingues à l'intérieur d'une même phrase.

- **Tickets, descriptions de MR, commentaires de revue, notes de mémoire : en français.** Les identifiants techniques restent tels quels dans leur langue d'origine : on écrit "le handler `AutoAcceptBidHandler` lève une `DomainException`", pas une traduction. Les messages d'erreur, les noms de tables, les commandes, les extraits de logs sont cités verbatim.

- **Code, noms de variables, commentaires dans le code : en anglais**, sur tous les dépôts, y compris le front et le mobile. Un commentaire de code en français est accepté dans le front et le mobile quand il explique une règle métier française (une règle de facturation, par exemple), parce que le glossaire est bilingue et que la traduction perdrait de la précision.

- **Sujets de commit** : anglais sur `halden-api` et `halden-infra`, français sur `halden-web` et `halden-driver-app`. C'est l'état historique de chaque dépôt et changer coûterait plus que ça ne rapporte. Voir la convention de message de commit du projet.

- **Documentation destinée aux intégrateurs externes** (OpenAPI, changelog public, guide des webhooks) : en anglais, parce que les intégrateurs sont dans cinq pays.

- **Alertes et runbooks** : en anglais, parce que l'astreinte ops compte deux personnes qui ne lisent pas le français.

Ce que l'équipe a explicitement refusé : traduire les messages d'exception de l'API en français (ils sont lus par des intégrateurs), et rédiger les notes de mémoire en anglais pour "être plus universel" (personne ne relit une note dans laquelle il a du mal à écrire).

Les collègues des bureaux allemand, polonais et néerlandais qui contribuent aux traductions du front écrivent leurs MR en anglais, et c'est très bien. La règle est pour l'équipe plateforme, pas pour ses contributeurs occasionnels.

Voir aussi [[naming-conventions-cross-stack]] pour le glossaire bilingue, qui est ce qui rend ce mélange lisible.
