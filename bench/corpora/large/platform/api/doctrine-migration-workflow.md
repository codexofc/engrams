---
name: doctrine-migration-workflow
description: How schema migrations are written, reviewed and run on halden-api (one migration per PR, no data in schema migrations, lock_timeout 5s, CONCURRENTLY for indexes)
type: reference
status: active
verified: 2026-05-20
---

# Migration workflow (Doctrine Migrations)

## Writing

- One migration per PR. If you need two, you need two PRs. Reviewers reject otherwise.
- `bin/console make:migration` then edit. Never ship the generated file as-is: it drops sequences it doesn't know about and reorders columns.
- Schema migrations never touch data. Data backfills go in a `bin/console app:backfill:<name>` command with `--batch-size` and `--dry-run`, run by hand after deploy. The reason is [[incident-2025-11-migration-lock-loads]].
- Every migration starts with `$this->addSql("SET lock_timeout = '5s'")`. If the `ALTER TABLE` can't get its lock in 5 s it fails, the deploy fails, and we retry when the long transaction is gone. Better than blocking the whole API.
- Index creation is `CREATE INDEX CONCURRENTLY`, which means the migration must declare `public function isTransactional(): bool { return false; }`. There is a PHPStan rule (`App\PHPStan\ConcurrentIndexRule`) that fails the build if `CONCURRENTLY` appears in a transactional migration.
- Adding a `NOT NULL` column: three steps across two deploys. Add nullable with default, backfill, then `SET NOT NULL` in a second migration. PostgreSQL 16 makes the `SET NOT NULL` fast only if a `CHECK (col IS NOT NULL) NOT VALID` was validated first, so the second migration does exactly that.
- Renaming a column is forbidden. Add, dual-write, migrate readers, drop. It took three sprints for `loads.pickup_date` to `loads.pickup_window_start` and nobody wants to do that again, but the alternative was downtime.

## Running

Migrations run as a Kubernetes `Job` from the ArgoCD `PreSync` hook, image `halden-api:<sha>`, command `bin/console doctrine:migrations:migrate --no-interaction --allow-no-migration`. The job has `activeDeadlineSeconds: 600`. If it exceeds that, the sync is aborted and the previous pods keep running.

The job connects directly to the primary through `DATABASE_URL_MIGRATIONS`, bypassing PgBouncer, because `CREATE INDEX CONCURRENTLY` and `SET lock_timeout` don't behave under transaction pooling. See [[postgres-connection-pool-pgbouncer]].

## Checking

- `bin/console doctrine:schema:validate --skip-sync` in CI, mapping must match.
- `bin/console doctrine:migrations:up-to-date` in CI against a database restored from the last prod dump (nightly job, `ci-db-restore`).
- The `migration-lint` CI step greps for `DROP TABLE`, `DROP COLUMN` and `ALTER COLUMN ... TYPE` and requires the PR to carry the label `schema-destructive`, which pings the on-call.

## Rollback

There is no `down()`. We write `down()` as `$this->throwIrreversibleMigrationException()`. Rolling back the app is fine because every migration must be compatible with the previous app version (expand and contract). Rolling back the schema is not supported.
