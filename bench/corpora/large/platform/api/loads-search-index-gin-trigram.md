---
name: loads-search-index-gin-trigram
description: Free-text load search uses a generated tsvector column with a GIN index plus a pg_trgm index on reference numbers, chosen over Elasticsearch in HF-1210
type: project
status: active
verified: 2026-03-05
---

# Recherche plein texte sur les chargements

Décision HF-1210 (décembre 2025) : pas d'Elasticsearch. Tout dans PostgreSQL, sur la table `loads`.

## Schéma

```sql
ALTER TABLE loads ADD COLUMN tsv tsvector
  GENERATED ALWAYS AS (
    setweight(to_tsvector('simple', coalesce(reference, '')), 'A') ||
    setweight(to_tsvector('simple', coalesce(pickup_city, '') || ' ' || coalesce(delivery_city, '')), 'B') ||
    setweight(to_tsvector('simple', coalesce(goods_description, '')), 'C')
  ) STORED;
CREATE INDEX CONCURRENTLY idx_loads_tsv ON loads USING gin (tsv);
CREATE INDEX CONCURRENTLY idx_loads_reference_trgm ON loads USING gin (reference gin_trgm_ops);
```

Dictionnaire `simple`, pas `french` ni `english` : les noms de villes et les références ne se stemment pas, et on a des chargements en cinq langues. Le stemming faisait matcher "Lyon" avec "lyonnais" et les dispatchers trouvaient ça bizarre.

Le `pg_trgm` sur `reference` sert au cas "je tape les 4 derniers caractères d'un numéro de chargement". Le tsvector ne fait pas de recherche par sous-chaîne. Le seuil `pg_trgm.similarity_threshold` est laissé à 0,3, on utilise `reference ILIKE '%' || :q || '%'` qui bénéficie de l'index trigram sans passer par la similarité.

## Requête

`App\Repository\LoadSearchRepository::search()` construit :

```sql
SELECT l.* FROM loads l
WHERE l.status = ANY(:statuses)
  AND (l.tsv @@ websearch_to_tsquery('simple', :q) OR l.reference ILIKE :like)
ORDER BY ts_rank(l.tsv, websearch_to_tsquery('simple', :q)) DESC, l.created_at DESC
LIMIT 50
```

`websearch_to_tsquery` plutôt que `plainto_tsquery` pour accepter les guillemets et le `-mot`. Un `:q` vide court-circuite vers la liste paginée classique.

## Mesures (prod, mars 2026, 2,2 M lignes après archivage)

- p50 : 18 ms, p95 : 95 ms, p99 : 240 ms.
- Taille de `idx_loads_tsv` : 310 Mo. `idx_loads_reference_trgm` : 88 Mo.
- Coût en écriture : l'`INSERT` dans `loads` est passé de 0,9 ms à 1,6 ms en moyenne. Acceptable, on fait environ 4 000 insertions par heure en pointe.

Les stats de la table sont sensibles, voir [[incident-2026-02-loads-search-timeout]].

## Ce qui reste ouvert

Pas d'autocomplétion des villes par cet index : elle passe par la table `cities` (référentiel géonames importé) avec un index trigram séparé. Les deux ne sont pas fusionnés parce que les villes des chargements sont saisies librement et contiennent des fautes.
