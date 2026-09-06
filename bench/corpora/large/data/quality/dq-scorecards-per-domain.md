---
name: dq-scorecards-per-domain
description: Depuis février 2026 chaque domaine a une fiche mensuelle de qualité, taux de succès, règles en échec, silences, coût, incidents attrapés et manqués, générée depuis dq.results, présentée à la revue, et ce que six mois de fiches ont montré, HF-4510
type: project
status: active
verified: 2026-07-07
---

# Fiches de qualité par domaine

## Pourquoi

Après le premier trimestre du framework ([[dq-framework-overview]]), la question de la direction était « est-ce que la qualité des données s'améliore », et la réponse était un tableau de bord Grafana que personne hors de l'équipe données n'ouvrait. HF-4510 (février 2026) a produit une fiche par domaine et par mois, une page, générée le premier du mois par `dq-runner scorecard --month 2026-06 --domain billing`, en markdown, postée dans le canal du domaine et jointe au compte rendu de la revue ([[dq-rules-review-feedback]]).

## Ce qu'il y a dessus

1. **Taux de succès** : exécutions `pass` sur exécutions totales, hors `info`. Un chiffre, et la variation par rapport au mois précédent.

2. **Règles en échec** au dernier jour du mois, avec la durée de l'échec. Une règle en échec depuis 20 jours est le premier sujet de la revue.

3. **Silences** posés dans le mois, avec ticket et durée. Un silence renouvelé apparaît en gras.

4. **Alertes** : nombre de `page`, nombre de `warn`, et pour chaque `page` une ligne « justifié / non justifié » remplie par l'astreinte après l'appel.

5. **Coût** : temps d'entrepôt consommé par les règles du domaine dans le mois, et les trois règles les plus coûteuses ([[dq-checks-runtime-cost]]).

6. **Attrapé / manqué** : les incidents du mois liés aux données du domaine, avec « la règle X l'a vu à T+N min » ou « aucune règle ne l'a vu, trouvé par Y ». Cette ligne est écrite à la main par le propriétaire, c'est la seule.

7. **Couverture** : tables `tier: 1` du domaine avec au moins une règle écrite par le propriétaire (pas seulement générée), sur le total.

### Exemple, `billing`, juin 2026

| Indicateur | Valeur | Mai |
|---|---|---|
| taux de succès | 99,71 % | 99,58 % |
| règles en échec au 30/06 | 0 | 1 (`payla_fees_vs_schedule`, 4 jours) |
| silences | 1 (`invoices_vs_payla_settlements`, rejeu HF-4590, 1 jour) | 0 |
| `page` | 1, justifié | 0 |
| `warn` | 6 | 11 |
| coût | 2 min 10 par jour | 2 min 40 |
| attrapé / manqué | 1 attrapé (14 règlements sans facture, T+0 au run de 06:30) | 0 / 0 |
| couverture | 9 / 9 tables | 8 / 9 |

## Ce que six mois de fiches ont montré

- **Le taux de succès ne dit rien.** Il est entre 99,5 et 99,8 % pour tous les domaines depuis le début, parce que la plupart des règles ne tombent jamais. Il reste sur la fiche parce que la direction l'a demandé, avec une note de bas de page qui dit de lire les lignes 6 et 7.

- **La couverture bouge.** `dispatch` est passé de 4 / 7 à 7 / 7 tables avec une règle écrite ; `product` est à 3 / 6 depuis février, faute de propriétaire, et la fiche le dit chaque mois, ce qui a fini par obtenir la promesse d'une personne pour septembre.

- **Les silences sont le meilleur indicateur.** Un domaine avec un silence renouvelé deux mois de suite a un problème qu'il ne règle pas. `pricing` a eu un silence de trois mois sur `bids_daily_volume` pour le cluster `ES-FR-north` (une lane qui a changé de nature après l'arrivée d'un gros expéditeur) ; la revue d'avril a fini par redéfinir le cluster plutôt que de renouveler.

- **La ligne attrapé / manqué est la seule qu'on relit.** Sur six mois et cinq domaines : 9 attrapés, 2 manqués (les deux de la fin 2025 et du printemps 2026, [[missed-2026-04-sek-invoices-summed-as-eur]] étant le second), 0 depuis mai. C'est le chiffre qu'on donne quand on demande si la qualité s'améliore.

## Ce qu'on a écarté

- Une note sur 100 par domaine. Ça produit un classement, et un classement produit des règles faciles qui passent toujours.

- Une fiche par table. Trop long, personne ne lit. Le détail est dans Grafana pour qui veut.

- Envoyer la fiche à la direction directement. Elle va au domaine, qui la présente lui-même à la revue trimestrielle avec ses mots. Les chiffres sont les nôtres, le récit est le leur.
