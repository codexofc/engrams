---
name: backup-encryption-and-key-custody
description: Backups leaving the racks are encrypted client-side with age-style keys per data class, the private keys live in the vault and in three sealed envelopes held by three people, the vault's own unseal shares are split 3-of-5, rotation yearly with the old keys kept for the retention window, and the drill checks that the envelopes open
type: reference
status: active
verified: 2026-05-27
---

# Backup encryption and who holds the keys

## Layers

1. **At rest in the racks**: the Stashbox appliances encrypt every drive with keys they manage in their controllers. A drive pulled from a chassis is unreadable. This protects against a drive leaving the building, not against anyone with appliance access.

2. **Leaving the racks**: everything written to the offsite disks ([[offsite-weekly-copy-contract]]) is encrypted by `offsite-sync` before it reaches the disk array, with our own keys. The datacentre provider carries ciphertext.

3. **Inside the buckets**: PostgreSQL WAL and base backups are encrypted by the operator's backup tool before upload (`encryption: age`, one recipient key per environment), so that `hf-pg-backups-prod` holds ciphertext even on the appliances. Documents are not encrypted at the object level (the API serves them through presigned URLs and would have to decrypt in the request path); the appliance encryption and the bucket policy are the controls there. Vault snapshots are encrypted by the vault itself. Velero backups are not encrypted beyond the appliance: they hold Kubernetes objects, of which the Secrets are projections from the vault, so the exposure is the same as the vault's and the vault is the thing protected.

## Keys

One key pair per data class and environment, `age`-style X25519, named `backup-<class>-<env>-<year>`:

| Key | Encrypts | Public key location | Private key custody |
|---|---|---|---|
| `backup-pg-prod-2026` | WAL and base backups | operator config, `halden-infra` | vault path `backups/keys/pg-prod-2026`, plus envelopes |
| `backup-offsite-prod-2026` | everything on the offsite disks | `offsite-sync` config | vault, plus envelopes |
| `backup-vault-prod-2026` | vault snapshots (the vault's snapshot encryption key) | vault | envelopes only, by construction |
| `backup-marmot-prod-2026` | marmot metadata backups | operator config | vault, plus envelopes |

Public keys are in Git; encryption needs nothing secret. Private keys are used only at restore time, which is rare and always a deliberate act by a named person.

## Custody

The private keys exist in two forms:

- In the vault, under a path readable by the `backup-restore` policy, which is bound to no application and to two named people's identities, with MFA. Reading a key writes an audit event and a message to the storage channel.

- On paper, in three sealed tamper-evident envelopes: one in the office safe, one at the datacentre provider's key deposit (part of the contract), one with a director at home. Each envelope holds all current private keys and the vault's unseal shares for the holder (see below). Envelopes are numbered; opening one is logged and it is replaced within a week.

The vault's own unseal is split 3-of-5 across five people (two on the platform team, one storage, one director, one on the data team). Three of them, or the three envelopes' shares, unseal a restored vault. The drill ([[restore-drill-2026-05]]) restored a vault from a snapshot and unsealed it with the envelope shares, which is the only test that the paper is correct; it was, and one envelope had a smudged character that took two attempts, so the envelopes now carry the shares in two encodings (hex and a word list).

## Why keys in the vault at all, if the vault is what we restore

Because most restores are not disaster restores. The February PITR ([[restore-2026-02-postgres-pitr-billing]]) needed the PostgreSQL backup key with the vault running fine; the on-call read it from the vault with their identity, the audit event fired, and no envelope was opened. The envelopes are for the day the vault is gone with the rack.

## Rotation

A new key pair per class every January (`-2027` keys created 2027-01, active for new backups from then). Old keys are kept, in the vault and in the envelopes, for as long as a backup encrypted with them can exist: 35 days for PITR, 52 weeks for offsite, so the `-2025` keys leave the envelopes in January 2027. A restore of an offsite disk from March 2026 in December 2026 needs the `-2026` key, and the runbook's first step is "which year is the backup from".

The rotation is a checklist in `halden-infra/storage/KEYS.md`: create, add to Git and configs, deploy, verify a backup written with the new key decrypts, print and seal new envelopes, retrieve and shred the ones with keys that expired.

## What is never done

- Keys in a repository, a CI variable, a config map, an env file, a ticket, or a chat message. The public half yes, the private half no. A private key seen in any of these is rotated the same day, and the rotation runbook has a "compromise" path that skips the January cadence.

- One key for everything. A class boundary means the offsite disks' key does not open the PostgreSQL backups in the buckets, so the provider's custody envelope, if opened, yields only what the provider already physically holds.

- Encrypting documents at the object level. Discussed in 2025; the presigned URL model makes it impossible without proxying every download, which the API was designed not to do, and the appliance encryption plus [[retention-by-data-class]] controls are the accepted position.
