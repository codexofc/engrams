---
name: edi-partners-overview
description: Five large shippers exchange EDIFACT (IFTMIN in, IFTSTA and INVOIC out) over AS2 through hf-edi-gateway, 3 100 loads a month; two more use the JSON path
type: reference
status: active
verified: 2026-06-09
---

# EDI partners: who, what, how much

Large shippers do not click in a web app; their transport management systems emit EDIFACT and expect EDIFACT back. `hf-edi-gateway` (own deployment, `integrations` namespace, PHP like the rest) is the translator. This note is the map; message-level details are in their own notes.

## Partners (June 2026)

| Partner (fictional name used in tickets) | Sector | Messages | Transport | Loads/month | Since |
|---|---|---|---|---|---|
| **Nordkarton** | packaging | IFTMIN in, IFTSTA out, INVOIC out | AS2 | 1 400 | 2024-09 |
| **Vestaflor** | fresh produce | IFTMIN in, IFTSTA out | AS2 | 800 | 2025-02 |
| **Bruma Retail** | retail distribution | IFTMIN in, IFTSTA out, INVOIC out | AS2 | 600 | 2025-06 |
| **Steelhaven** | metals | IFTMIN in, IFTSTA out | AS2 | 200 | 2025-11 |
| **Kalmarine** | chemicals | IFTMIN in, IFTSTA out, INVOIC out | AS2 | 100 | 2026-03 |
| two mid-size shippers | | JSON equivalents, see [[edi-api-json-alternative]] | HTTPS | 350 | 2026 |

3 100 EDIFACT loads a month is 11 % of platform volume but about 25 % of revenue: these are the large accounts. Nordkarton's quirks alone have a note ([[edi-partner-nordkarton-quirks]]).

## Messages

- **IFTMIN** (transport instruction), inbound: creates or updates a load. Mapping in [[edifact-iftmin-mapping]].

- **IFTSTA** (status), outbound: load events (accepted, picked up, in transit, delivered, cancelled) as status codes. Mapping and the code list fights in [[edi-status-messages-iftsta]].

- **INVOIC**, outbound: our commission invoice to the shipper as a structured message, for the three partners whose accounts payable ingest it. [[edi-invoice-invoic-outbound]].

- **CONTRL** and **APERAK**: syntax and application acknowledgements, both directions. Ours are generated; theirs are parsed and drive the reject workflow ([[edi-rejects-handling]]).

Syntax: UN/EDIFACT D.96A for four partners, D.01B for Kalmarine. We generate and parse both; the differences that matter are in segment qualifiers, not structure.

## Transport

AS2 over HTTPS for all five ([[as2-transport-setup]]). SFTP was the first transport and is gone ([[edi-sftp-legacy-transport]]).

## Partner profile

`edi_partners (id, code, organization_id, syntax_version, sender_id, receiver_id, as2_id, as2_cert_pem, transport, status_code_set, invoice_enabled, reconciliation_contact, timezone, created_at)`. One row per partner; the mapping tables per partner are described in [[edi-mapping-tables-location]]. `organization_id` links the EDI partner to the shipper organization on the platform, so a load created from an IFTMIN belongs to that organization and their web users see it like any other load.

## Volumes and health

- Inbound IFTMIN per day: about 150, peak 400 on Monday mornings.

- Outbound IFTSTA per day: about 900 (6 per load on average).

- Rejects: 2.1 % of inbound in May 2026, target under 2 %, details and causes in the rejects note.

- Daily reconciliation between what they think they sent and what we have: [[edi-reconciliation-daily]].

## Ownership

Two engineers in the integrations team, sharing with telematics. The commercial owner of each account is the escalation path for anything that needs the partner's IT to change something, which is slow: the median time for a partner to change a mapping on their side has been 6 weeks. That number shapes every decision in this project: we adapt on our side whenever we can. Preferences in [[edi-team-preferences]].
