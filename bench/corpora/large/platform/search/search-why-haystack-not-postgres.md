---
name: search-why-haystack-not-postgres
description: Décision HF-3010 : la recherche transporteur quitte PostgreSQL pour haystack, pourquoi HF-1210 reste vrai pour le chargeur, chiffres et refus
type: project
status: active
verified: 2026-03-27
---

# Pourquoi la recherche transporteur est sortie de PostgreSQL (HF-3010)

En décembre 2025, HF-1210 avait tranché : pas d'Elasticsearch, le plein texte des chargements reste dans PostgreSQL avec une colonne `tsvector` et un index GIN. Un mois plus tard, HF-3010 a créé un cluster de recherche dédié. Les deux décisions tiennent ensemble parce qu'elles ne parlent pas de la même recherche.

## Deux recherches

**La recherche chargeur** (tableau de répartition) : un chargeur cherche **ses** chargements par référence, ville, description. Quelques milliers de lignes par organisation, filtre sur `org_id` d'abord, plein texte ensuite. PostgreSQL fait ça très bien, et HF-1210 reste la règle pour cette recherche.

**La recherche transporteur** (place de marché) : un transporteur cherche parmi **tous** les chargements `OPEN` et `BIDDING` (30 000 à 60 000 selon l'heure) ceux qui partent dans un rayon de son point, vers une zone, à des dates, pour ses types de véhicule, triés par pertinence (distance, prix au km, fraîcheur, correspondance avec ses habitudes). Facettes sur les pays et les types de véhicule. C'est une requête géographique, multi-critères, classée, sur tout le corpus, plusieurs fois par seconde.

## Ce qui n'allait plus

Mesures de janvier 2026 sur la recherche transporteur en PostgreSQL (PostGIS pour le rayon, index GiST, tri par une expression) :

- p50 320 ms, p95 1,8 s, p99 4,1 s aux heures de pointe (7 h à 9 h). Le front affichait un spinner et les transporteurs rafraîchissaient, ce qui empirait.

- 40 % de la charge CPU du primaire entre 7 h et 9 h venait de cette seule route, et elle concurrençait les écritures (offres, transitions) sur les mêmes tables.

- Le tri par pertinence était une expression SQL de 60 lignes, impossible à faire évoluer sans casser le plan d'exécution, et pas de facettes sans une seconde requête.

- Les réplicas de lecture ne suffisaient pas : le rayon plus le tri plus la pagination profonde (les transporteurs vont jusqu'à la page 10) ne tenaient pas dans un plan indexé.

L'incident de février 2026 sur `GET /v2/loads/search` (plan séquentiel après un archivage massif) a touché la recherche chargeur, pas celle-ci, mais il a rappelé qu'on n'avait plus de marge sur le primaire.

## Ce qu'on a choisi

Un cluster de recherche dédié, moteur de type OpenSearch, nommé `haystack` ([[search-haystack-cluster-layout]]), alimenté par les événements de chargement ([[search-indexing-pipeline]]). Lecture seule pour la place de marché ; la vérité reste PostgreSQL, et un clic sur un résultat lit le chargement dans l'API, qui répond `load_not_open` si l'index a du retard.

Résultats après bascule (mars 2026, [[search-query-latency-slo]]) : p50 35 ms, p95 110 ms, p99 280 ms. Charge CPU du primaire aux heures de pointe divisée par deux.

## Ce qu'on a refusé

- **Mettre la recherche chargeur aussi dans haystack.** Elle marche, elle est cohérente à la transaction (un chargeur qui vient de créer un chargement le voit tout de suite), et la cohérence compte plus que la vitesse pour cet usage.

- **Un service géré chez un fournisseur.** Le corpus est petit (60 000 documents actifs, 2 Go avec les réplicas), le cluster tient sur trois machines modestes, et on voulait la maîtrise des versions et des paramètres. Réévalué si le corpus dépasse dix fois ça.

- **Un moteur vectoriel ou sémantique.** La recherche transporteur est structurée : lieux, dates, types, prix. Le peu de texte libre (marchandise) se traite par des synonymes ([[search-synonyms-city-names]]). Pas de besoin identifié.

- **Réécrire les offres et les transitions pour lire haystack.** Rien d'écrit ne dépend de l'index. Si haystack tombe, la recherche est dégradée (repli sur une requête PostgreSQL simplifiée sans classement, avec une bannière), et tout le reste fonctionne.

## Ce que la décision a coûté

Deux personnes pendant huit semaines, trois machines, un pipeline d'indexation à surveiller, et une nouvelle classe d'incidents (le retard d'indexation, [[search-incident-2026-03-index-lag]]). On savait que ça viendrait et ça est venu.
