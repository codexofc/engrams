---
name: n-plus-one-loads-list-fix
description: GET /v2/loads did 1+3N queries (shipper, pickup address, bid count per row), fixed in HF-1275 with fetch joins and a lateral subquery, 50 rows went from 160 queries to 3
type: project
status: active
verified: 2025-12-18
---

# N+1 sur la liste des chargements

## Constat (novembre 2025)

Le profiler Symfony sur `GET /v2/loads?limit=50` en staging : 161 requêtes SQL, 340 ms. Le toolbar Doctrine signalait 3 requêtes répétées 50 fois :

1. `SELECT ... FROM shippers WHERE id = ?` (lazy load de `Load::$shipper` au moment de sérialiser `shipper.name`)
2. `SELECT ... FROM addresses WHERE id = ?` (idem pour `pickupAddress`)
3. `SELECT count(*) FROM bids WHERE load_id = ?` (le sérialiseur appelait `$load->getBids()->count()`, et comme la collection est `EXTRA_LAZY`, Doctrine fait un `COUNT` par appel)

Personne ne l'avait vu parce qu'en dev la base a 30 chargements et que 90 requêtes de 0,3 ms, ça ne se voit pas.

## Correctif (HF-1275)

Dans `LoadRepository::createListQueryBuilder()` :

```php
$qb->select('l', 's', 'pa', 'da')
   ->from(Load::class, 'l')
   ->join('l.shipper', 's')
   ->join('l.pickupAddress', 'pa')
   ->join('l.deliveryAddress', 'da');
```

Pour le nombre d'offres, pas de `fetch join` sur `bids` (ça multiplie les lignes et casse la pagination par curseur). À la place, une colonne calculée via `addSelect` avec une sous-requête, et Doctrine la remonte dans un tableau `[0 => Load, 'bidCount' => int]`. Le `LoadListItemNormalizer` prend ce tableau plutôt que l'entité.

Résultat : 3 requêtes (la liste, plus deux pour les documents rattachés qui restent lazy parce qu'on ne les affiche pas dans la liste), 28 ms.

## Garde-fou

`App\Tests\Api\QueryCountAssertions::assertMaxQueries(int $n, callable $fn)` compte via un `DebugStack` DBAL. `LoadsListTest` fait `assertMaxQueries(5, ...)` sur une base de 200 chargements. Toute PR qui ajoute un champ sérialisé et déclenche un lazy load fait échouer ce test, ce qui est le but.

Le seuil est 5 et pas 3 pour laisser une marge. La règle non écrite : si tu dois monter le seuil, tu expliques pourquoi dans la PR.

## Ce qu'on n'a pas fait

`Doctrine\ORM\Query::HINT_FORCE_PARTIAL_LOAD` et les entités partielles : trop fragiles, un accès à un champ non chargé donne `null` en silence. Les DTO de lecture (`SELECT NEW`) ont été envisagés et rejetés pour l'instant parce que le normaliseur JSON est écrit contre les entités et qu'on ne voulait pas le doubler. Si la liste grossit encore en champs, on y reviendra.

Voir aussi [[doctrine-dql-vs-native-sql-advice]].
