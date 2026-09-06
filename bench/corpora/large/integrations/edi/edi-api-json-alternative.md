---
name: edi-api-json-alternative
description: Mid-size shippers use POST /v2/edi/instructions, JSON with IFTMIN semantics, synchronous validation with the same 12 reject reasons, webhooks for status
type: project
status: active
verified: 2026-04-16
---

# The JSON alternative to IFTMIN (HF-2133)

Not every shipper that wants "EDI" has an EDI department. Two mid-size shippers in early 2026 had a TMS that could emit JSON over HTTPS and wanted the same integration promise: instruct a load from their system, get statuses back, no manual entry. Building EDIFACT into their TMS would have cost them months. So the gateway grew a second front door with the same back.

## Endpoint

`POST /v2/edi/instructions` on the platform API, authenticated with an organization API key (IAM project rules) scoped `load.create` and `load.update`. The body is a JSON document validated against `schemas/transport-instruction-v1.json`:

```json
{
  "function": "original",
  "reference": "VF-2026-000812",
  "pickup":   {"site_code": "HUB-LYON", "slot": {"from": "2026-04-16T06:00:00+02:00", "to": "2026-04-16T08:00:00+02:00"}},
  "delivery": {"address": {...}, "contact": {"name": "...", "phone": "..."}, "slot": {...}},
  "goods":    {"pallets": 12, "weight_kg": 8400, "adr": null, "temperature_c": null},
  "instructions": "Quai 4",
  "target_price": {"amount": 890, "currency": "EUR"}
}
```

The field names are the IFTMIN semantics in plain words: `function` is `BGM.1001`, `reference` is `BGM.1004`, `pickup.site_code` is `LOC+9` with a partner code, and so on. The document is converted to the **same internal `TransportInstruction` object** that `IftminToLoadMapper` produces from EDIFACT ([[edifact-iftmin-mapping]]), and everything after that (idempotency, mapping tables, overrides, publication hold, reconciliation) is shared. There is one mapper from `TransportInstruction` to `Load`, two parsers in front of it.

## Synchronous answers

Where EDIFACT gets an APERAK minutes later, JSON gets the answer in the response:

- `201` with the load id and `warnings[]` (the same `WARN:` texts as the EDIFACT path).

- `409` with `{"reason": "duplicate_reference", ...}` or `422` with one of the other reject reasons from [[edi-rejects-handling]], same twelve codes, same phrasing. The partner's developer reads the same documentation page as the EDIFACT partner's EDI team.

Timezones: ISO 8601 with offset, mandatory. No "local time of the partner" rule; the JSON path refuses a slot without an offset (`422 slot_timezone_missing`, a reason that only exists here).

## Statuses

No IFTSTA. The shipper subscribes to the standard load webhooks (`load.accepted`, `load.picked_up`, `load.delivered`, `load.eta_updated`...), documented by the webhooks team. The event vocabulary is the platform's, not the EDIFACT code list, and the two JSON shippers were happy with that. `edi_status_code_sets` is not consulted for them.

## Reconciliation

None needed daily: the response is the acknowledgement. The monthly report ([[edi-reconciliation-daily]]) still runs for them because month-end invoicing disputes do not care about the transport.

## Why it lives in the EDI project

Because the hard part of "EDI" was never the syntax. It is the partner reference idempotency, the site code tables, the replacement semantics, the publication hold, the reject vocabulary. All of that is shared. The JSON parser is 300 lines; the EDIFACT parser is 2 400; the mapper behind both is 1 800 and is where the incidents happened ([[incident-2026-03-edi-duplicate-loads]] applied to both paths on the same day).

## Numbers

Two shippers, 350 loads a month in April 2026, reject rate 0.9 % (lower than EDIFACT: synchronous errors get fixed by the developer at integration time). Median integration time for the second shipper: 9 working days from API key to production, against 6 to 10 weeks for an EDIFACT partner.

## Not done

- A JSON equivalent of INVOIC. The two shippers take PDF invoices by email like everyone else; the billing project's e-invoicing work may cover this later.

- Bulk instructions (many loads in one call). Asked once; one call per load is fine at 350 a month, and it keeps the reject semantics simple.
