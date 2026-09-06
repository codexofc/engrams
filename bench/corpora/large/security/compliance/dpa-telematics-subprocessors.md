---
name: dpa-telematics-subprocessors
description: Trakko and Geolyx are listed sub-processors with signed DPAs; Trakko needed a 2026 amendment to stop 3 year retention of positions for analytics
type: project
status: active
verified: 2026-02-18
---

# Telematics providers as sub-processors (HF-2083)

The telematics integration notes describe how positions come in from Trakko and Geolyx. This note is about the paperwork: when a carrier's vehicle is tracked by one of those providers and we pull its positions, the provider is processing personal data (a driver's whereabouts) and we need to know under what terms.

## The two relationships

It is not symmetrical, and that took a while to see.

- **Geolyx**: the carrier has a contract with Geolyx for their own fleet. We receive positions because the carrier authorised Geolyx to push them to us. Geolyx is the carrier's processor, we are a recipient. Our DPA with Geolyx covers only what Geolyx does for **us**: the webhook infrastructure, the API logs, their support access to our integration. Signed 2025-12-04, EU-only processing, 30 day log retention on their side.

- **Trakko**: about 60 % of the trackers reporting through Trakko are units we bought and installed for small carriers under our own subscription (the "Halden tracker" offer). For those, we are the controller and Trakko is our processor in full. Their standard terms said they retained position data for **3 years** for "service analytics and product improvement", which is not an instruction we gave them. An amendment signed 2026-01-22 limits retention of our data to 90 days on their side (matching [[data-retention-matrix]]), forbids secondary use, and requires deletion certificates on request. The commercial side was harder than the legal side: their pricing assumed the data.

## What we ask of every telematics sub-processor

Recorded in the sub-processor list at `https://docs.halden.example/legal/subprocessors` and in `subprocessors.yaml` in `halden-legal`:

- Processing location (country list). Both are EU only. A third candidate provider in 2025 was dropped at this step because their support team had access from outside the EU without a transfer mechanism.

- Retention of raw positions on their side, and whether they derive anything from it. Trakko's ETA product uses aggregated positions across all customers; the amendment allows aggregation only after removal of vehicle identifiers.

- Breach notification delay: 48 h to us, which lets us hold our 72 h to authorities ([[breach-notification-72h-procedure]]).

- Sub-sub-processors: named list, notice on change. Trakko uses two hosting providers, Geolyx one.

- Deletion at end of contract within 30 days, with a written confirmation.

## Where drivers and carriers see this

The carrier DPA ([[dpa-carriers-template]]) refers to the sub-processor list, and the in-app tracking consent screen for drivers names the provider used by their vehicle ("positions collected by Trakko on behalf of Halden Freight" or "shared by your employer's Geolyx account"). The wording difference is deliberate: in the Geolyx case the driver's employer made the choice, not us.

## Open item

A carrier asked in February 2026 whether we could give them the Trakko deletion certificate for their vehicles when they leave. We can request one; Trakko produces them per account, not per vehicle, so the answer is a written statement from us plus the certificate at our own contract end. Recorded in the DPA variants for that carrier.
