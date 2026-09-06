---
name: search-incident-2026-05-shard-hotspot
description: Mai 2026, p99 triplé trois jours : un routage par pays de départ concentrait 45 % des documents sur un shard, retiré, alerte de déséquilibre ajoutée
type: project
status: active
verified: 2026-06-10
---

# Incident de mai 2026 : le shard chaud (HF-3138)

## Ce qu'on a fait de travers

Le 2026-05-11, une modification livrée avec de bonnes intentions : router chaque document vers un shard selon `pickup.country` (`routing = pays`), avec l'idée qu'une recherche filtrée sur un pays n'interrogerait qu'un shard au lieu de six, donc moins de travail total. Le raisonnement tenait pour la charge cumulée. Il oubliait deux choses.

## Ce qui s'est passé

- 11 mai 14:00 : déploiement de l'indexeur avec le routage, réindexation complète dans `loads-v6` ([[search-reindex-runbook]]), bascule des alias à 14:40.

- 12 mai 07:30 : `SearchLatencyP99High` (280 ms de seuil) se déclenche à l'heure de pointe. p99 à 750 ms, p50 inchangé à 35 ms. Pas de page (seuil de page à 1 s), notification seulement. L'astreinte regarde, ne voit rien d'anormal sur le cluster (CPU moyen 40 %), suppose la charge du mardi.

- 13 mai : même chose, p99 à 820 ms. Un des deux de la paire recherche regarde la répartition : le shard 2 porte 41 000 documents sur 90 000, les shards 0 et 4 en portent 3 000 chacun. Le CPU du nœud du shard 2 est à 95 % pendant que les autres dorment. La moyenne à 40 % cachait tout.

- 14 mai 10:00 : retour au routage par défaut (hash de `_id`), réindexation dans `loads-v7`, bascule à 10:20. p99 revenu à 260 ms à 10:25.

## Les deux oublis

**Les pays ne sont pas équirépartis.** 45 % de nos chargements partent d'Allemagne et de France ; ces deux pays sont tombés dans le même shard par le hash du routage. Avec six shards et une douzaine de valeurs de routage très inégales, un déséquilibre était presque certain.

**La plupart des recherches ne filtrent pas sur un pays.** Une recherche par rayon de 150 km autour de Strasbourg touche deux pays ; une recherche sans zone de livraison touche tous les shards. Le gain espéré (une recherche, un shard) concernait 15 % des requêtes. Les 85 % autres interrogeaient les six shards comme avant, et attendaient le plus lent, qui était désormais toujours le shard 2. Une requête distribuée va à la vitesse de son shard le plus chargé ; en concentrant les documents on a concentré la latence.

## Pourquoi trois jours

La métrique regardée était la moyenne du CPU du cluster. Le tableau de bord n'avait pas la répartition par shard ni le CPU par nœud en évidence. Et le p50 n'avait pas bougé, ce qui a fait penser à un problème de longue traîne côté clients. Le retour Ravello côté support a été reçu la même semaine, on a d'abord cru à un lien.

## Correctifs

- Routage par défaut, définitivement. Un commentaire dans le code de l'indexeur renvoie à ce ticket pour la prochaine personne qui aura l'idée.

- Tableau de bord `haystack` : panneau « documents par shard » et « CPU par nœud de données », avec une alerte `HaystackShardImbalance` quand le plus gros shard dépasse 2,5 fois la médiane. Aurait sonné à 14:41 le 11 mai.

- Le runbook de réindexation exige de vérifier la répartition par shard avant la bascule des alias (`GET _cat/shards/loads-v*?v&s=docs`), en une ligne.

- La procédure de revue des changements d'indexation demande une mesure sur la copie de production restaurée d'un snapshot ([[search-haystack-cluster-layout]]) pour tout ce qui touche au routage, au nombre de shards ou aux analyseurs. Trente minutes qui auraient évité trois jours.

## Ce qu'on a appris

- Six shards pour 1,2 Go n'ont qu'un but : répartir le CPU des requêtes. Tout ce qui déséquilibre la répartition détruit ce but.

- Une moyenne de cluster ne dit rien d'un cluster à six nœuds. On regarde le max.

- Le p50 stable et le p99 qui triple, c'est la signature d'un nœud lent, pas d'un problème client.

## Chiffres

Trois jours, p99 entre 700 et 850 ms aux heures de pointe, p50 inchangé, aucun ticket client direct (les transporteurs ont attendu un peu plus), un SLO mensuel entamé de 30 % ([[search-query-latency-slo]]).
