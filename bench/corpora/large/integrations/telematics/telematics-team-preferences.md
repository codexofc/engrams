---
name: telematics-team-preferences
description: The telematics pair wants every provider quirk in the adapter with a dated comment, server time over device time, a counter per stage, a fixture per incident
type: user
status: active
verified: 2026-05-12
---

# Préférences de l'équipe intégrations télématiques

Deux personnes, un fournisseur de boîtiers, un fournisseur de flotte, une application, 2 900 véhicules. Ce qu'on tient à faire d'une certaine façon, pour que le prochain ne recommence pas.

- **Toute bizarrerie fournisseur vit dans l'adaptateur, et nulle part ailleurs.** Passé `RawPosition`, le pipeline ne sait pas d'où vient la position. Si une étape a besoin de `if ($provider === 'trakko')`, c'est que l'adaptateur n'a pas fini son travail. Une seule exception assumée : le compteur par fournisseur sur chaque étape, qui est de l'observabilité, pas de la logique.

- **Chaque bizarrerie a un commentaire qui dit quand elle a mordu.** `// ignition is "On" on the 2024 batch, seen 2025-06-12` vaut mieux qu'un `strtolower` muet. Dans six mois, quelqu'un voudra simplifier, et le commentaire lui dira ce qui cassera. La note [[trakko-api-contract-quirks]] est la version longue de ces commentaires.

- **L'heure serveur, jamais l'heure de l'appareil, quand les deux sont disponibles.** Et quand elles divergent, on garde les deux. Leçon de [[incident-2026-02-trakko-timestamp-drift]], et l'équipe mobile l'avait apprise avant nous.

- **Un compteur par étape et par issue.** Pas de log par position (41 millions par jour), des compteurs. « Il manque des positions » a une réponse en trente secondes sur le tableau de bord : quelle étape, quel fournisseur, depuis quand. C'est l'architecture de [[position-ingestion-pipeline]] et on ne la négocie pas.

- **Chaque incident devient une fixture rejouable.** La rafale Geolyx, la dérive Trakko, un lot mal signé : tous dans le harnais ([[telematics-integration-test-harness]]), rejoués avant chaque changement du pipeline. Un incident sans fixture est un incident qu'on aura deux fois.

- **La contrainte en base, pas seulement le cache.** La déduplication a un index unique derrière elle depuis novembre 2025 ; on ne reviendra pas dessus pour gagner un index.

- **On ne stocke pas ce qu'on n'a pas de raison de montrer.** Pas de vitesse, pas de conducteur, pas de carburant, pas de position hors mission. Ce n'est pas seulement la conformité : c'est aussi ce qui garde la table et le pipeline simples.

- **Le push avant le poll.** Un fournisseur qui pousse nous donne 8 s de latence, un fournisseur qu'on sonde nous en donne 40. Quand un fournisseur offre les deux, on prend le push et on garde le poll comme rattrapage.

- **On dit aux dispatchers l'âge de la position**, pas seulement la position. Un point vieux de 12 minutes affiché comme frais est un mensonge, voir [[dispatchers-feedback-position-age]].

- **On écrit les tickets en anglais** quand ils concernent un fournisseur (on les leur transmet parfois tels quels), en français sinon.

- **On refuse les fonctionnalités « puisqu'on a les données »**. Score de conduite, pauses, éco-conduite : la réponse est dans la note du projet conformité sur le fondement juridique, et on n'a pas à la réécrire à chaque fois.

Ce qui nous fatigue : les tickets « le camion est au mauvais endroit » qui sont des mappings ([[tracker-vehicle-mapping]]) neuf fois sur dix. L'écran de support existe, le lire d'abord.
