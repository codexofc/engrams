---
name: gdpr-data-map
description: The data map lists 11 personal data categories across 47 tables, 6 object stores and 4 vendors, generated from PersonalData annotations by compliance:datamap
type: reference
status: active
verified: 2026-03-27
---

# GDPR data map

The data map answers "where is personal data, whose is it, why do we have it, how long, who else sees it". It is generated, not written: `bin/console compliance:datamap --format md` walks the Doctrine metadata and produces `docs/compliance/datamap.md`, which is committed and diffed in every release. This note explains the model and records what the generated map does not say.

## Annotations

Every column holding personal data carries `#[PersonalData(category: ..., subject: ..., basis: ...)]`. Categories are a closed enum:

| category | examples |
|---|---|
| `identity` | name, company role |
| `contact` | email, phone, postal address of a person |
| `credentials` | password hash, PIN hash, MFA secrets |
| `identifiers` | user id, device id, IdP subject |
| `location` | driver positions, stop detections |
| `documents` | licence scans, insurance certificates, POD photos with signatures |
| `financial` | IBAN of a sole-trader carrier, payout history |
| `communication` | chat messages, support notes |
| `behavioural` | login history, product analytics events |
| `professional` | driver licence categories, ADR certificate |
| `verification` | Verifid verdicts, sanctions screening result |

`subject` is `shipper_user`, `carrier_user`, `driver`, `staff`, `contact_person` (someone named on a load who has no account) or `prospect`. `basis` is `contract`, `legal_obligation`, `legitimate_interest` or `consent`, and a `legitimate_interest` requires an entry in the balancing test register (see [[records-of-processing-register]]).

`PersonalDataAnnotationTest` fails when a column whose name matches `/(name|email|phone|address|iban|birth|licence|position|lat|lon)/i` has no annotation and is not in `config/compliance/not_personal.yaml` (which lists, for example, `warehouses.address`, a business address).

## Beyond the database

The generated map covers Postgres. The rest is maintained by hand in `docs/compliance/datamap-stores.md`:

- **Object storage**: `hf-documents` (licences, insurance, PODs), `hf-avatars`, `hf-exports` (customer exports, 7 day TTL), `hf-audit-archive`, `hf-support-attachments`, `hf-edi-archive` (EDIFACT messages, which contain contact persons of the shipper's customers).

- **Search index**: loads and carriers, contains contact names, rebuilt from Postgres, 24 h max lag for erasure.

- **Warehouse**: pseudonymised copies, see the data platform's own retention notes; the data map only records what leaves the platform and under which pseudonymisation.

- **Logs**: `user_id` and `organization_id` as structured fields, never names or emails (a Loki rule alerts on an `@` inside a log line, which fires about once a month on a stack trace).

- **Vendors**: Payla (payment data of shippers and carriers), Verifid (identity documents during the check, not retained by us), the two telematics providers (positions, see the telematics integration notes and [[dpa-telematics-subprocessors]]), the transactional email provider (emails, names).

## What the review found in March 2026

- `loads.delivery_contact_phone` was annotated `contact` / `shipper_user`, but the person is the shipper's customer, someone with no relationship to us. Re-annotated as `contact_person` with basis `legitimate_interest`, added to the balancing register, and given a shorter retention (see [[data-retention-matrix]]).

- Chat messages between dispatcher and driver were not in the map at all: the table `messages` was created in 2025 without annotations, and its columns did not match the name regex. Added, and the regex now also matches `body|content|text` when the table is not in the not-personal list.

- The EDI archive holds contact persons of third parties in plain EDIFACT for 10 years (see the EDI integration notes on archival). Flagged; the decision is recorded in [[dpo-feedback-position-data]]'s companion discussion and the retention matrix.

## How to use it

Before a DSAR (see [[dsar-handling-runbook]]), before an erasure, before answering a security questionnaire ([[vendor-security-questionnaire]]), and before adding a vendor. If a question about personal data cannot be answered from the map, the map is wrong and fixing it comes first.

## Reading the generated map

`docs/compliance/datamap.md` is grouped by category, then by table, one line per column:

```
## location
- position_events.lat, .lon        subject: driver        basis: legitimate_interest (BT-03)   retention: positions_raw_90d
- trip_summaries.start_commune     subject: driver        basis: legitimate_interest (BT-03)   retention: trip_summaries_24m
- stops.arrived_at                 subject: driver        basis: contract                      retention: loads_10y
```

Each line links the column to a retention rule id from [[data-retention-matrix]] and, for legitimate interest, to a balancing test id. A column whose annotation names a retention rule that does not exist fails the build, which is how the matrix and the map stay in step.

## Counts in March 2026

| Category | Columns | Tables |
|---|---|---|
| identity | 31 | 14 |
| contact | 44 | 19 |
| credentials | 6 | 3 |
| identifiers | 58 | 47 |
| location | 9 | 3 |
| documents | 7 (object keys) | 4 |
| financial | 11 | 5 |
| communication | 5 | 2 |
| behavioural | 12 | 4 |
| professional | 8 | 3 |
| verification | 6 | 2 |

197 annotated columns. The number that surprised people is `contact` at 44: every place that stores a phone or an email, including the six copies of "who to call at the delivery site" spread across loads, stops, assignments and the EDI archive. Consolidating those is on the list, not started.

## How a DSAR uses it

`compliance:dsar:export` iterates the map: for each annotated column, it selects rows where the subject matches the request (by `user_id`, `driver_id`, or the phone/email for contact persons without an account) and writes them into the export under the category heading. A column without an annotation is invisible to the export, which is the second reason the annotation test exists: a missed annotation is not only a missing line in a document, it is a DSAR answer that is incomplete.

## Vendors and what they hold

| Vendor | Categories | Retention on their side | DPA |
|---|---|---|---|
| Payla | financial, identity, contact | their regulatory retention (10 years for transaction records) | signed 2024, reviewed 2026-01 |
| Verifid | verification (documents only during the check) | 30 days then deleted, verdict kept by us | signed 2025 |
| Trakko | location, identifiers | 90 days since the 2026 amendment | see [[dpa-telematics-subprocessors]] |
| Geolyx | location (relayed), identifiers | 30 days of logs | idem |
| email provider | contact, communication (rendered emails) | 30 days | signed 2024 |

The map lists Verifid as holding `verification` even though they delete after 30 days, because for those 30 days they hold identity documents we never see, and a DSAR received in that window has to reach them.
