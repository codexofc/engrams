---
name: records-of-processing-register
description: The register is a linted YAML in halden-legal with 23 activities; legitimate-interest entries carry a balancing test, rendered to PDF monthly
type: reference
status: active
verified: 2026-04-20
---

# Register of processing activities

The register is what an authority asks for first. Ours is `halden-legal/register/activities.yaml`, 23 entries in April 2026, rendered to `docs/compliance/register.pdf` by a monthly pipeline and reviewed with the data map ([[gdpr-data-map]]) twice a year.

## One entry

```yaml
- id: A07
  name: Real-time position tracking of assigned vehicles
  role: controller
  purpose: Show shippers where their load is and compute ETAs
  subjects: [driver]
  categories: [location, identifiers]
  sources: [mobile_app, telematics_trakko, telematics_geolyx]
  recipients: [shipper_users_of_the_load, telematics_provider_as_processor]
  transfers_outside_eu: none
  retention_ref: positions_raw_90d, trip_summaries_24m
  basis: legitimate_interest
  balancing_test: BT-03
  security_ref: TOM-2026-01
  owner: integrations
```

Fields are validated by `compliance:register:lint` against a schema: known category and subject enums (the same as the data map annotations), `retention_ref` must exist in the retention matrix ([[data-retention-matrix]]), `basis: legitimate_interest` requires `balancing_test`, `role: processor` requires a `controller` field naming the customer category and the DPA version.

## The 23 activities (grouped)

- **Accounts and access** (A01 to A04): user accounts, staff accounts, authentication logs, security audit trail.

- **Marketplace** (A05, A06): loads and bids, carrier verification (KYC through Verifid, sanctions screening).

- **Execution** (A07 to A10): position tracking, proof of delivery, dispatcher-driver messaging, ETA computation.

- **Money** (A11 to A14): invoicing, payments and payouts through Payla, dunning, self-billing.

- **Support and quality** (A15 to A17): support tickets, impersonation sessions, customer satisfaction surveys.

- **Analytics** (A18, A19): product analytics (consent, see [[cookie-consent-and-analytics]]), pricing and demand models on pseudonymised data.

- **Integrations** (A20, A21): EDI exchanges with shippers (we are processor for their contact persons), integrator API access.

- **Corporate** (A22, A23): staff HR data held by the HR tool (we are controller, the tool is processor), recruitment.

## Balancing tests

Six activities rest on legitimate interest and each has a two-page balancing test `BT-01` to `BT-06` in `halden-legal/register/balancing/`. The one that took the longest is `BT-03`, position tracking, summarised in [[driver-position-legal-basis]]. The structure of each: the interest, why it is legitimate, why the processing is necessary (what we tried that was less intrusive), the impact on the person, the safeguards, and the conclusion. The DPO signs each with a date; a test older than 24 months fails the lint with a warning.

## Processor entries

For A20 (EDI) and the driver-data part of A05 and A08 we are the processor. Those entries name the controller category ("shipper under EDI agreement v2", "carrier under DPA v3", see [[dpa-carriers-template]]) and the instructions we follow, which are the service terms. A processor entry has no balancing test because the basis is the controller's problem.

## Keeping it honest

- A new table with `#[PersonalData]` annotations whose `activity` attribute references an unknown `A` id fails `PersonalDataAnnotationTest`. The annotation was extended with `activity:` in HF-2140 for exactly this reason.

- Adding a vendor requires adding it to `recipients` of at least one activity, otherwise `compliance:register:lint` complains that the sub-processor list ([[dpa-telematics-subprocessors]] for the telematics ones) has an orphan.

- The monthly PDF is stored with a hash in `hf-compliance/register/`, so we can show what the register said on a given date.
