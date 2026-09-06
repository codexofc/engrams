---
name: environments-dev-staging-prod
description: Three environments since the 2025 simplification, local dev (docker compose), staging (namespace platform-staging, nightly prod-like data, continuous deploy from main) and prod (platform-prod, deploy by tag), plus ephemeral preview environments per MR for the web front only
type: reference
status: active
verified: 2026-05-04
---

# Environments

Three, plus ephemeral previews for the front. The former four-tier layout with a `preprod` is described in [[environments-old-preprod]].

## Local (`dev`)

`docker compose up` in each repository. The API compose file brings PostgreSQL 16 with PostGIS, Redis, RabbitMQ, an S3-compatible object store, a mail catcher and the API itself with Xdebug available on demand (`XDEBUG_MODE=debug` in `.env.local`). The web front runs `vite dev` on the host against the local API or against staging (`HF_API_URL` in `.env.local`, staging is the default because most front work does not need API changes). The driver app points at staging by default, with a debug menu to switch.

Seed data: `bin/console app:seed:dev` creates 3 shippers, 5 carriers, 30 loads in every status, and the well-known accounts (`dispatcher@shipper-a.test`, PIN `000000` is refused so the seeded drivers use a random PIN printed at the end of the command). The seed is deterministic (fixed random seed) so screenshots and tests match between machines.

## Staging (`platform-staging`)

- Namespace `platform-staging` on the same cluster as production (see the ops notes on cluster layout for the isolation argument).

- URLs: `api.staging.halden.example`, `app.staging.halden.example`, `tiles` shared with prod.

- Deployed continuously from `main` by ArgoCD: every merge produces an image tagged with the commit SHA, and the staging application tracks `main`. Typical delay merge to live: 6 minutes.

- Data: restored every night at 02:00 from the last production backup, anonymised by `app:anonymise` (names, e-mails, phone numbers, addresses replaced with generated ones keeping the city, documents replaced by a placeholder PDF). Anonymisation takes 25 minutes. Anything created in staging during the day is gone the next morning, by design. People who need persistent test data keep a seed script.

- Third parties: mail goes to a catcher with a web UI, push goes to the real providers but only to devices registered on staging builds, webhooks go to real subscriber URLs only if the subscription has `staging_allowed = true`, otherwise to a sink.

- Secrets: separate from production, same names. A staging secret leaking gives access to anonymised data only.

- Feature flags: separate table, usually everything on.

## Production (`platform-prod`)

- Deployed by tag (`api-2026.19`, `web-2026.19`), see [[branching-and-pr-flow]]. The tag updates the image reference in the `halden-infra` repository through a small pipeline job, and ArgoCD syncs with manual approval for the API (a human clicks after checking the migration job plan) and automatically for the front.

- Promotion window: 10:00 to 16:00 on working days, never Friday afternoon, never during the month-end invoicing run (last working day, 06:00 to 09:00). Exceptions with two approvals.

- Every promotion is announced in the team channel with the tag, the list of tickets (generated from the squash commit subjects) and the name of the person watching the dashboards for the next 30 minutes.

## Preview environments (front only)

Each MR on `halden-web` gets `mr-<number>.preview.halden.example`, a static build served by a shared deployment that routes by hostname to the MR's build directory in object storage. Points at the staging API. Created by the CI in 90 s, deleted when the MR closes. Used by reviewers and by product for visual checks. There is no preview for the API: an API change is reviewed from the code and tested in staging after merge, which has been enough so far, and an API preview would need a database per MR.

## Differences that have bitten us

- Staging has 80 000 loads after anonymisation (the restore keeps only 90 days), production has 2.2 million. A query that is fast in staging can be slow in production, which is the origin of the "rows in prod" line in the migration MR template.

- Staging shares the cluster's ingress and certificates with production. An ingress misconfiguration in staging manifests can affect production routing. The ops team reviews every ingress change for that reason.

- Staging has one replica of everything, production has 45 API pods. Anything that assumes a single process (a file lock, an in-memory cache used as a lock) works in staging and breaks in production. This happened with the first version of the outbox relay.

- Time: staging has no month-end invoicing run. The invoicing job is tested by running it by hand in staging on the anonymised data, which is on the checklist of every release that touches invoicing.

## What each environment is for

| Question | Where |
|---|---|
| Does my code work | local |
| Does it work with real-shaped data and the other components | staging |
| Does it hold under real load | production, watched, with a flag |
| Does the UI look right | preview environment, then staging |

## The anonymisation contract

`app:anonymise` is the one command that decides whether staging is safe to give to every engineer, so its scope is written down and tested:

- Replaced: person names, e-mail addresses, phone numbers, street lines of addresses (city, postcode and country kept), IBANs, company registration numbers, driver PINs (reset to a random value per driver, printed nowhere), API client secrets, webhook secrets and subscriber URLs (replaced by the sink URL).

- Kept: load references, prices, dates, cities, vehicle types, statuses, scores, everything that makes the data look real for testing.

- Documents: every object in the restored bucket prefix is replaced by one of three placeholder PDFs, and `documents.sha256` is recomputed so the integrity check still passes.

`app:anonymise --verify` runs after the pass and greps the whole database dump for the 200 most common French, Polish and German surnames, for any string matching an e-mail or an E.164 phone number, and fails the restore job if anything is found. It found something once, in December 2025: a phone number typed by a dispatcher inside `loads.goods_description`. The free-text fields (`goods_description`, `stops.instructions`, `messages.body`) are now scrubbed by a regex pass for phone numbers and e-mails, and the verify step covers them. The full restore plus anonymisation plus verify takes 25 minutes and finishes by 02:40.

## Access per environment

- Local: everything is on the developer's machine, no access question.

- Staging: every engineer has `edit` on the namespace and read access to the database through the bastion with the `staging_readonly` role. Write access to the staging database is not granted by default, the seed scripts run through the API's commands. The preview environments share the staging API and therefore the same data.

- Production: `view` on the namespace for engineers, `edit` for the on-call during their week, database access only through the `support_readonly` role with a 30 minute session that has to be requested in the ops channel with a reason. Every session is logged with the reason. Two people have standing write access to the production database, and they are the ones who run the backfill commands.

The difference between staging and production access is the whole point of the nightly anonymised restore: it is what allows staging to be open.
