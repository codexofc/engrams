---
name: product-spec-template-preferences
description: Product managers' spec preferences: one page in the epic, non-goals mandatory, figures with their source, written reviews
type: user
status: active
verified: 2026-02-13
---

Préférences partagées par les quatre product managers (billing, pricing, onboarding, dispatch), écrites après une discussion en janvier 2026.

- **Une spécification tient sur une page dans le ticket epic**, avec les sections dans cet ordre : problème, ce qu'on sait (chiffres, appels, tickets support), ce qu'on ne fait pas (non-objectifs), ce qu'on mesure, déploiement (flag, audiences, note de version). Pas de document séparé, pas de présentation.

- **Français ou anglais, au choix de celui qui écrit**, sans mélanger dans une même section. Les identifiants de code, noms d'événements et de flags restent en anglais quelle que soit la langue du texte.

- **Un chiffre vient avec sa requête ou sa source** (le nom de la table de l'entrepôt et la période, ou la note de mémoire qui le contient). Un chiffre sans source est retiré en relecture. Les entonnoirs se citent sur cohorte close, jamais sur la semaine en cours.

- **Les non-objectifs sont obligatoires** et ce sont eux qu'on relit en premier. Une spécification sans non-objectif est renvoyée.

- **Le déploiement décrit le flag** ([[feature-flags-convention]]) avec son unité de randomisation et sa date d'expiration, et le texte de la note de version pour chaque audience ([[release-notes-rules]]).

- **Les maquettes sont des liens**, pas des captures collées dans le ticket, pour qu'elles restent à jour.

- **Les revues de spécification se font par écrit**, en commentaire dans le ticket, dans les 48 heures. Une réunion n'est convoquée que si deux commentaires se contredisent.

- **Les décisions** (y compris ce qu'on a refusé et pourquoi) sont dans l'epic, pas dans le fil de discussion. Le fil disparaît, l'epic reste.

