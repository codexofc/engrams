---
name: doctrine-dql-vs-native-sql-advice
description: When to write DQL, when to drop to native SQL with ResultSetMapping, and when to bypass the ORM entirely with DBAL on halden-api
type: feedback
status: active
verified: 2026-03-19
---

# DQL, SQL natif ou DBAL

Trois niveaux, et le bon réflexe est de descendre d'un cran dès que le niveau du dessus se met à faire des contorsions.

## DQL par défaut

Lecture d'entités, jointures simples, filtres sur colonnes indexées. Tout ce qui retourne des entités que le code métier va manipuler (`$load->dispatchTo()`). Le `QueryBuilder` reste lisible tant qu'on n'a pas plus de trois `join` et une sous-requête.

Signal d'alarme : dès qu'on écrit `->addSelect('(SELECT COUNT(...)...) AS HIDDEN ...')` ou qu'on cherche une fonction DQL custom pour `websearch_to_tsquery`, on est en train de forcer.

## SQL natif avec `ResultSetMappingBuilder`

Quand la requête a besoin d'une fonction PostgreSQL (`@@`, `<->` de pg_trgm, `jsonb_path_query`, `LATERAL`, fenêtres) mais que le résultat doit être des entités hydratées. `LoadSearchRepository::search()` (voir [[loads-search-index-gin-trigram]]) est le bon exemple : SQL natif, `addRootEntityFromClassMetadata(Load::class, 'l')`, et Doctrine hydrate des `Load` normaux.

Attention au `addJoinedEntityFromClassMetadata` : il faut sélectionner toutes les colonnes de l'entité jointe avec le préfixe d'alias, et l'oubli d'une colonne donne une hydratation partielle silencieuse. Le `ResultSetMappingBuilder::generateSelectClause()` évite l'erreur, l'utiliser toujours.

## DBAL pur (`Connection::fetchAllAssociative`)

Reporting, exports, tout ce qui retourne des lignes agrégées que personne ne va modifier. `App\Reporting\*` n'a pas le droit d'importer `EntityManagerInterface`, il reçoit `Doctrine\DBAL\Connection`. PHPStan le vérifie (`ReportingNoOrmRule`).

Aussi pour les écritures en masse : `INSERT ... ON CONFLICT` de l'allocateur de numéros de facture ([[invoice-numbering-sequence]]), le backfill par lots. Passer par `persist()` pour 50 000 lignes, c'est 50 000 entités dans l'`UnitOfWork` et un `flush()` qui prend 30 secondes et 800 Mo. Si on doit vraiment persister beaucoup d'entités, `flush()` puis `clear()` tous les 500.

## Ce qu'on ne fait pas

- Pas d'`EXTRA_LAZY` sur les collections sans savoir ce qu'il coûte, voir [[n-plus-one-loads-list-fix]].
- Pas de fonctions DQL custom pour émuler PostgreSQL. On en avait 6 (`JSON_GET_TEXT`, `TSMATCH`, ...), toutes retirées en HF-1301. Le SQL natif est plus lisible que `TSMATCH(l.tsv, :q) = TRUE`.
- Pas de `Query::getResult()` sans `setMaxResults()` sur une table de plus de 100 000 lignes. Il n'y a pas de garde-fou automatique, c'est en revue.

## Comment choisir en 10 secondes

Si le résultat est modifié ensuite → DQL. Si c'est lu, affiché et oublié, et que la requête a une fonction PG → natif avec RSM. Si c'est un agrégat ou un export → DBAL.
