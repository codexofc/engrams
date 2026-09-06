---
name: postgres-connection-pool-pgbouncer
description: halden-api goes through PgBouncer in transaction mode (pool 60, max_client_conn 2000), which forbids prepared statements, LISTEN and session-level SET
type: project
status: active
verified: 2026-06-11
---

# PgBouncer devant PostgreSQL

Depuis HF-980 (septembre 2025), l'API ne parle plus directement au primaire. Un PgBouncer en mode `transaction` tourne comme sidecar-less Deployment (`pgbouncer-api`, 2 replicas) dans le namespace `platform-prod`.

Chiffres actuels :

- `default_pool_size = 60` par base, `max_client_conn = 2000`.

- `server_idle_timeout = 120`, `server_lifetime = 3600`.

- Le primaire a `max_connections = 200`, dont 60 réservés au pool API, 20 aux workers Messenger (pool séparé `pgbouncer-workers`), 10 aux migrations et au reste pour les humains et les exporters.

Avant PgBouncer : 45 pods `php-fpm` avec 12 workers chacun = 540 connexions potentielles, et on a touché `max_connections` deux fois en août 2025 pendant les pics du matin (7 h 30, quand les transporteurs ouvrent l'app).

## Ce que le mode transaction interdit

- **Prepared statements côté serveur**. `PDO::ATTR_EMULATE_PREPARES => true` est forcé dans `config/packages/doctrine.yaml`. Doctrine DBAL 4 avec pdo_pgsql accepte ça sans problème, mais un `dbal.connection` custom qui oublie l'option produit `prepared statement "pdo_stmt_00000001" does not exist`. Ça nous est arrivé sur le worker d'export comptable.

- **`SET` de session**. `SET lock_timeout` dans une migration doit passer par `DATABASE_URL_MIGRATIONS` (connexion directe). `SET LOCAL` dans une transaction est correct.

- **`LISTEN/NOTIFY`**. Le worker de notifications temps réel a sa propre connexion directe, `DATABASE_URL_LISTEN`, une seule par pod, voir [[webhook-delivery-outbox]].

- **Advisory locks de session**. `pg_advisory_lock()` est interdit, `pg_advisory_xact_lock()` est le seul autorisé. PHPStan a une règle qui cherche la chaîne.

- **`search_path`** par session : tout est dans `public`, on ne change pas.

## Diagnostic

```
kubectl -n platform-prod exec deploy/pgbouncer-api -- psql -p 6432 -U pgbouncer pgbouncer -c 'SHOW POOLS;'
```

Colonnes utiles : `cl_waiting` (clients qui attendent une connexion serveur, doit rester à 0) et `maxwait` (secondes d'attente du plus vieux client). L'alerte `PgBouncerClientsWaiting` part quand `cl_waiting > 5` pendant 2 minutes.

Un `maxwait` qui grimpe sans que le nombre de requêtes grimpe signifie presque toujours une transaction longue qui garde une connexion serveur : `SELECT pid, now() - xact_start, query FROM pg_stat_activity WHERE state = 'idle in transaction' ORDER BY 2 DESC;` sur le primaire.

## Lien avec Doctrine

L'`EntityManager` ouvre une transaction dès le premier `flush()`, et pas avant. Une requête HTTP qui fait un `SELECT` puis attend 800 ms un service externe puis `flush()` ne tient une connexion serveur que pendant le `SELECT` (autocommit) et pendant la transaction du `flush()`. C'est le principal gain de PgBouncer pour nous. La règle qui en découle : jamais d'appel HTTP sortant entre `beginTransaction()` et `commit()`.
