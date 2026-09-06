---
name: staging-environment-cost-cut
description: Décembre 2025 à février 2026, staging ramené de 3 nœuds permanents plus 8 VM Skyvale à 1 nœud et des VM à la demande éteintes la nuit et le week-end, la copie de prod réduite à un échantillon de 1 %, −2 100 EUR par mois, ce qui a cassé et ce qu'on a gardé, HF-4725
type: project
status: active
verified: 2026-03-10
---

# Réduire le coût de staging

## Le point de départ

Staging était une réplique de prod à l'échelle un tiers : 3 nœuds du pool réservés en permanence (1 650 EUR par mois d'amortissement et de colocation), 8 VM Skyvale pour les runners CI et deux environnements de démonstration commerciale allumés 24 h sur 24 (2 400 EUR par mois), et une copie complète des données de prod restaurée chaque nuit (les 60 TB d'objets n'étaient pas copiés, mais la base PostgreSQL de 400 GB oui, et 1 % de l'entrepôt). Personne n'avait décidé cette taille ; elle était celle de 2024, quand staging servait aussi de recette pour trois équipes en parallèle.

En décembre 2025, l'usage mesuré : les nœuds staging à 6 % de CPU en moyenne, 20 % au pic (les déploiements de fin d'après-midi), 0 % la nuit et le week-end. Les runners CI à 40 % d'usage en journée, 0 la nuit. Les démonstrations commerciales utilisées 11 fois dans le mois, 2 heures à chaque fois.

## Ce qu'on a fait (HF-4725)

1. **Un nœud au lieu de trois.** Les requêtes de staging, après le dimensionnement de [[rightsizing-2025-q4-requests-limits]] appliqué aussi à staging, tenaient sur un nœud de 32 vCPU avec de la marge. Les deux autres sont retournés au pool, ce qui a contribué à ne pas commander les nœuds prévus en 2026 ([[reserved-capacity-decision-2026-01]]).

2. **Les runners CI à la demande.** Un autoscaler de runners sur Skyvale : 1 runner permanent, jusqu'à 10 à la demande, extinction après 20 minutes d'inactivité. Heures de VM mensuelles : de 5 800 à 1 900. La file d'attente au pic du matin est passée de 0 à 90 secondes (le temps de démarrer une VM), ce que les équipes ont accepté après une semaine de grogne.

3. **Les démonstrations allumées à la demande.** Un bouton dans le back-office commercial démarre l'environnement de démo (3 minutes) et l'éteint 4 heures après ou à la demande. Coût : de 900 à 90 EUR par mois. Les commerciaux ont demandé un rappel la veille des rendez-vous ; il existe.

4. **La copie de prod réduite à 1 %.** La base staging est reconstruite chaque nuit à partir d'un échantillon : 1 % des transporteurs et des expéditeurs avec tous leurs chargements, enchères, factures et documents (les références, pas les objets), plus tous les comptes internes, le tout anonymisé par le même pipeline qu'avant. De 400 GB et 2 h 40 de restauration à 6 GB et 8 minutes. Le stockage des instantanés de staging est passé de 2 TB à 60 GB.

5. **Extinction la nuit et le week-end** du nœud staging lui-même : non. Un nœud bare-metal éteint ne coûte que son électricité (environ 40 EUR par mois de moins), et le redémarrage de tous les pods chaque matin coûtait 20 minutes de flottement à la première équipe qui déployait. Le nœud reste allumé.

## Mesuré

| | Novembre 2025 | Mars 2026 |
|---|---|---|
| nœuds staging | 3 | 1 |
| heures de VM Skyvale par mois (CI plus démos) | 5 800 + 1 400 | 1 900 + 180 |
| coût staging et CI dans la table par service | 3 900 EUR | 1 800 EUR |
| temps de restauration nocturne | 2 h 40 | 8 min |
| attente moyenne d'un runner CI au pic | 0 s | 90 s |
| tickets « staging ne marche pas » par mois | 4 | 6 le premier mois, 2 ensuite |

Économie : 2 100 EUR par mois, dont 1 800 côté Skyvale et 300 côté stockage et pool. Elle apparaît dans les lignes CI et staging de [[per-service-cost-table-q2-2026]].

## Ce qui a cassé

- **Les tests de performance.** Deux équipes lançaient des tests de charge sur staging à trois nœuds et les comparaient d'un mois sur l'autre. Sur un nœud, les chiffres ne se comparent plus à l'historique. Décision : les tests de charge tournent sur un nœud à la demande du pool, réservé une journée, avec le même profil que la prod, et l'historique repart de janvier 2026.

- **L'échantillon à 1 % a manqué des cas.** Un bug de facturation multi-devises n'était reproductible que sur un expéditeur suédois, et l'échantillon n'en contenait aucun. L'échantillonnage est maintenant stratifié : au moins 5 transporteurs et 5 expéditeurs par pays et par devise, puis 1 % du reste. 9 GB au lieu de 6.

- **Une démo commerciale n'a pas démarré** parce que le bouton avait été déplacé dans le back-office et que le commercial ne l'a pas trouvé, cinq minutes avant un rendez-vous. Le rappel de la veille inclut maintenant le lien direct.

## Ce qu'on a gardé volontairement

- Un environnement staging permanent, pas des environnements éphémères par branche. La question a été posée ; la réponse est que trois équipes déploient sur staging dix fois par jour et qu'un environnement partagé qui ressemble à la prod attrape des problèmes d'intégration que des environnements isolés ne voient pas. Le coût d'un nœud est le prix de cette ressemblance.

- La restauration nocturne depuis la prod plutôt que des données de test synthétiques : l'échantillon anonymisé a la forme réelle des données, avec les cas bizarres, et c'est ce qui rend staging utile.

- Les mêmes manifestes que la prod, à la taille près. Un staging qui diverge de la prod dans sa configuration ne teste plus la prod.
