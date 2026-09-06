---
name: finops-monthly-review-feedback
description: Ce que neuf revues mensuelles des coûts ont appris: les gros gains étaient des erreurs, le matériel se discute avec l'usage, refacturation refusée
type: feedback
status: active
verified: 2026-07-08
---

# La revue mensuelle des coûts : retours

Depuis octobre 2025, le premier mercredi du mois, une heure, la responsable plateforme, le responsable données, une personne de la finance et la personne de permanence du mois ([[costs-reviewer-preferences]]). Neuf revues au moment d'écrire.

## Ce qui marche

- **Une heure suffit si la table est prête.** Les deux premières revues ont duré deux heures parce qu'on construisait la table pendant la réunion. Depuis que `finops-report` la produit le premier du mois et que la réconciliation ([[vendor-invoices-reconciliation]]) est faite avant, l'heure tient et finit souvent en avance.

- **Les gros gains étaient des erreurs, pas des optimisations.** Les tuiles ([[egress-finding-map-tiles-2025-11]]), les journaux en `DEBUG` oubliés ([[logging-cost-reduction-2026]]), les nœuds pris « pour un pic » et gardés dix mois, les requêtes de CPU dimensionnées en 2023. Aucun n'a demandé d'ingéniosité, tous ont demandé qu'on regarde. Le premier trimestre a rapporté 7 300 EUR par mois de baisse ([[infra-cost-overview-2026]]) presque uniquement comme ça. Le second trimestre, avec les erreurs corrigées, a rapporté 900. C'est normal et c'est dit à la direction pour qu'elle n'attende pas 7 000 par trimestre.

- **Une demande de matériel se discute avec l'usage du demandeur ouvert.** Le troisième nœud GPU ([[gpu-vs-cpu-inference-cost]]) et les deux nœuds de calcul du budget 2026 ont été refusés ou annulés en regardant les requêtes, le p95 et le calendrier des pics du demandeur, pas en discutant du principe. Personne n'a mal pris un refus qui montre ses propres chiffres.

- **La ligne `shared` visible.** La tentation de répartir le plan de contrôle et la surveillance au prorata a été forte (« comme ça le total par service fait 100 % »). On ne l'a pas fait, et deux fois la ligne `shared` a été le sujet de la revue (les journaux, les VM de CI) alors qu'elle aurait été invisible étalée sur douze services.

## Ce qui a changé d'avis

- **Les alertes d'anomalie en `warn` plutôt qu'en `page`**, sauf les SMS. La première version paginait sur tout au-dessus de 2× ; trois pages nocturnes pour des réécritures planifiées de l'entrepôt en un mois ont suffi ([[cost-anomaly-alerts]]).

- **L'engagement à 80 % plutôt qu'à 100 %** chez Skyvale ([[reserved-capacity-decision-2026-01]]). La première proposition était 100 % « pour maximiser la remise » ; la finance a fait remarquer qu'une remise sur de la capacité vide n'est pas une remise.

- **La refacturation interne, toujours refusée, mais mieux argumentée.** La finance l'a proposée trois fois. La réponse qui a fini par convaincre : la table sert à décider, et une équipe qui reçoit une facture optimise la facture (les étiquettes, le périmètre) plutôt que le coût. La finance a obtenu en échange la répartition par centre de coût pour le budget, calculée une fois par an depuis la table, ce qui est ce dont elle avait besoin.

## Ce qui reste difficile

- **La saisonnalité de l'électricité.** La ligne colocation varie de 900 EUR entre janvier et août avec la climatisation, et chaque hiver quelqu'un demande pourquoi elle monte. La réponse est dans le compte rendu de l'hiver précédent, et on la ressort.

- **Les coûts de l'équipe données dans deux tables.** L'entrepôt publie son propre chiffre (14 200) avec sa propre formule ; la table par service donne le même total par construction, mais les sous-lignes ne se recoupent pas exactement (leur « part Kafka » est notre ligne torrent répartie). On a choisi de ne pas unifier : leur note parle à leurs analystes, la nôtre à la direction, et le total est le même.

- **Les économies négatives.** L'IP dédiée Courrix (+600 par mois), le second nœud GPU (+1 150), les tiroirs de stockage (+460) sont des dépenses décidées pour de bonnes raisons et elles apparaissent comme des hausses. Le compte rendu les liste avec leur raison dans une section « ce qu'on a choisi de payer », pour que la revue ne devienne pas un exercice où toute hausse est un échec.

## L'ordre du jour, fixe depuis janvier

1. La table du mois et les lignes qui ont bougé de plus de 10 % (10 minutes).

2. Les anomalies du mois et leurs acquittements (10 minutes).

3. Le rapport de dimensionnement des requêtes, les dix premiers (10 minutes).

4. Les demandes en cours : matériel, renouvellements, engagements (15 minutes).

5. L'économie du mois, mesurée, et la prochaine nommée (10 minutes).

6. Ce qu'on a choisi de payer (5 minutes).

Le compte rendu est le diff de `finops/savings.md`, `finops/anomalies.md` et, s'il y a lieu, `finops/costs.yaml`.
