---
name: as2-transport-setup
description: All EDIFACT partners use AS2 at as2.halden.example: signed and encrypted S/MIME, signed MDN, one cert pair per partner rotated with a 30 day overlap
type: reference
status: active
verified: 2026-03-27
---

# AS2 transport

AS2 is what every large shipper's EDI department already runs, so it is what we run. The implementation is the `As2Receiver` and `As2Sender` classes in `hf-edi-gateway` around a PHP S/MIME library; no third-party AS2 gateway product. The SFTP transport it replaced is described in [[edi-sftp-legacy-transport]].

## Endpoint

`https://as2.halden.example/as2/inbound`, one URL for all partners. The partner is identified by the `AS2-From` header, matched against `edi_partners.as2_id`. Unknown `AS2-From`: 403 and a log line, no MDN.

Outbound goes to the partner's URL stored in `edi_partners.as2_url`. Three partners restrict inbound by source IP, so outbound uses the static egress the webhooks team set up (their notes have the addresses).

## Message security

- **Signed** (SHA-256) and **encrypted** (AES-256-CBC) S/MIME in both directions, no exception. One partner asked for unencrypted-but-signed "because our test environment"; the test environment got a certificate instead.

- **Certificates**: one pair per partner per direction. Their public certificate to verify their signatures and encrypt to them: `edi_partners.as2_cert_pem`, plus `as2_cert_next_pem` for rotation. Our private key: in the vault under `edi/as2/<partner>/key`, one key per partner so that a compromise scopes to one relationship. Our public certificates are self-signed, 3 years, distributed to the partner by the account's commercial owner through their EDI onboarding form.

- **Rotation**: at 34 months, generate the new pair, send the new public certificate to the partner, set `as2_cert_next_pem` when they send theirs. During the **30 day overlap** the receiver accepts signatures from either of the partner's certificates and the sender signs with our old key until the partner confirms they installed the new one (`edi_partners.as2_our_cert_switched_at`). Reminder ticket created automatically at 33 months. First rotation cycle done for Nordkarton in 2027 will be the real test; the procedure was rehearsed on staging with a partner's test endpoint in 2026-02.

## MDN

We request a **signed, synchronous MDN** for every outbound message and send one for every inbound. The MDN's `Received-Content-MIC` is compared to our computed MIC; mismatch is treated as not delivered.

- Outbound with no MDN or a failed MDN within **30 minutes**: retry with the same `Message-ID`, 5 times over 6 hours, then the message goes to `edi_outbound_failed` and the daily reconciliation ([[edi-reconciliation-daily]]) flags it as `status_unacked`.

- Inbound: we send the MDN **after** the raw message is written to `hf-edi-archive` (10 year retention, the compliance project's matrix) and a row exists in `edi_messages` with `status = 'received'`. Parsing and mapping happen after the MDN. A parse failure therefore produces a CONTRL, not a missing MDN; the partner's AS2 layer sees success, their application layer sees the reject, which is the correct separation.

## Duplicates at the transport layer

`Message-ID` of inbound messages is unique-indexed in `edi_messages`. A retried inbound with a known `Message-ID` gets an MDN again (the partner did not see our first one) and is not processed again. This is the transport-level guard; the application-level one is `BGM.1004` per partner (see [[edifact-iftmin-mapping]] and the incident where the two guards were not enough, [[incident-2026-03-edi-duplicate-loads]]).

## Operations

- `edi:as2:test <partner>`: sends a signed, encrypted test message (EDIFACT `UNB..UNZ` with a `TEST` indicator in `UNB.0035`) and reports MDN status and round-trip time. Run after every certificate change and every partner-side firewall change.

- Alerts: inbound 4xx or 5xx rate above 5 % over 15 minutes for one partner; outbound MDN failure above 3 in a row for one partner; certificate expiry under 45 days.

- Logs contain `AS2-From`, `Message-ID`, sizes, MIC match, timings. Never the payload; the payload is in the archive with restricted access because IFTMIN messages contain third-party contact persons.

## Partner-side quirks

- Nordkarton's AS2 stack sends `Content-Transfer-Encoding: binary` and rejects `base64` inbound. Per-partner setting `as2_transfer_encoding`.

- Kalmarine's endpoint returns 200 with an empty body instead of an MDN for about 1 % of messages, then sends the MDN asynchronously 2 to 10 minutes later. We accept an async MDN for a message that is still in the 30 minute window even though we asked for sync.

- Steelhaven's certificate chain includes an intermediate we must present back in our MDN signature or their verifier fails. `as2_include_chain = true`.

## Onboarding checklist for a new partner's AS2

What the account owner sends to the partner's EDI team, and what we need back. Kept here because it is asked for every time.

We send:

- our AS2 identifier (`HALDEN-AS2-PROD`, staging: `HALDEN-AS2-STG`), the inbound URL, our public certificate for this partner (PEM), our egress IP addresses, the MDN mode we expect (signed, synchronous), the algorithms (SHA-256, AES-256-CBC), and a test window.

We need back:

- their AS2 identifier, their inbound URL, their public certificate (PEM, with the chain if any), whether they restrict by source IP, their MDN mode, and a contact for the test window.

Then, in order: `edi_partners` row with `status = 'testing'`, certificates loaded, `edi:as2:test` in both directions, a test IFTMIN from them that we reject on purpose (to check the CONTRL path), a test IFTSTA from us, and only then `status = 'active'`. Median duration of this transport step alone: 4 working days, of which 3 are waiting for the partner's firewall change.

## Sizes and limits

- Inbound message size limit: 5 MB after decryption; the largest real IFTMIN seen is 41 KB, the largest count file 900 KB. A message over the limit gets a 413 and an alert, because it means either an abuse attempt or a partner sending something they should not.

- Concurrency: the receiver is stateless and runs on 2 pods; the partners' total rate is under 1 message per second at peak.

- The archive bucket grows by about 1.2 GB a month; at 10 years of retention that is a manageable 150 GB.
