---
name: ci-old-runners
description: The cloud CI runners used until March 2026: 40-minute lanes, no dev kit, 1 900 EUR a month, replaced by self-hosted machines
type: reference
status: archived
superseded_by: [[ci-pipeline-layout]]
verified: 2025-11-10
---

Jusqu'en mars 2026 la CI tournait sur des runners loués à l'heure chez un fournisseur cloud.

- Lane code : 40 minutes en moyenne, parce que chaque exécution repartait d'une machine vide (cache de compilation téléchargé à chaque fois, 6 Go).
- Pas de kit de développement de la cible basse dans le cloud, donc pas de test de fumée sur la vraie machine ; les régressions spécifiques à la cible étaient trouvées au playtest.
- Coût : environ 1 900 EUR par mois, plus le transfert des assets (3 Go par build).

Les quatre machines du studio ([[ci-pipeline-layout]]) ont coûté l'équivalent de sept mois de location et ramènent la lane code à 11 minutes. Le runner cloud reste configuré comme secours, désactivé, au cas où la salle machine tombe.
