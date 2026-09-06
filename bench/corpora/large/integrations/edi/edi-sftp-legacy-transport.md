---
name: edi-sftp-legacy-transport
description: From 2024-09 to 2025-03 EDIFACT files went through SFTP drop folders polled every 5 min with no receipt acknowledgement; replaced by AS2 after two lost files
type: reference
status: archived
superseded_by: [[as2-transport-setup]]
verified: 2025-09-05
---

# SFTP transport (2024 to March 2025)

The first EDI partner (Nordkarton) started on SFTP because it was what both sides could set up in a week. It lasted six months. The current transport is [[as2-transport-setup]].

## What it was

- An SFTP server `edi-sftp.halden.example` with one account per partner, key authentication, chrooted to `/in` and `/out` directories.

- Partners dropped IFTMIN files as `<sender>_<timestamp>_<seq>.edi` in `/in`. A cron polled every 5 minutes, moved each file to `/in/processed/` after parsing, and wrote the load. We dropped IFTSTA files in `/out`; the partner polled at their own interval (Nordkarton: 15 minutes) and deleted what they took.

- Deduplication by file name only. A partner that re-uploaded a file with a new timestamp in the name produced a duplicate load; this happened twice before the `BGM.1004` reference guard existed.

- No acknowledgement at the transport level. A file that we picked up and failed to parse simply stayed in `/in/failed/`, and nobody on the partner side knew until the month-end comparison. CONTRL and APERAK were generated as files in `/out`, which the partner's poller did not always read.

- Time to load: 5 minute poll, so 0 to 5 minutes plus parsing; time to status at the partner: up to 15 minutes plus their processing.

## Why it went

- **Two lost-file disputes** (December 2024 and February 2025): the partner's upload was interrupted mid-transfer, our poller picked up a truncated file, parsing failed, the file sat in `/in/failed/`, and 6 then 11 loads were missing at month end. The partner's logs said "uploaded"; ours said "failed"; both were right.

- No standard way to say "received and readable" at the moment of transfer. AS2's signed MDN is exactly that.

- Large partners' EDI departments consider SFTP a legacy transport and asked for AS2 at onboarding; Vestaflor (February 2025) refused SFTP outright, which set the migration date.

## Migration

Nordkarton moved to AS2 on 2025-03-18 after a 3 week parallel run where both transports carried the same messages and a script compared them (identical every day). The SFTP server was kept read-only for 60 days, then decommissioned; the account keys were revoked and the host is gone. The `edi_messages.transport` column still shows `sftp` for messages before that date, which is the only trace left.

## What carried over

- The idea that a message must be stored raw before anything is done to it (the truncated-file incident made it obvious).

- The per-partner directory of overrides, which began as a per-partner config file next to the SFTP account.
