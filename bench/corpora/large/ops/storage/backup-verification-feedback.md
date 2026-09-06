---
name: backup-verification-feedback
description: What a year of restores taught us about verifying backups: checksums are not restores, the inventory lies until exercised, and five checks per backup line
type: feedback
status: active
verified: 2026-06-04
---

# Verifying backups: what we learned

Written after the May drill ([[restore-drill-2026-05]]), from the three real restores of the year and the two drills. Applies to every line of [[backup-inventory-and-retention]].

## Things we believed and stopped believing

- **"The backup job succeeded" means there is a backup.** It means a job wrote bytes somewhere. The `hf-exports-prod` bucket had a green replication dashboard because the dashboard showed the buckets that had replication configured; the one without it was not on the dashboard. A success signal that cannot fail for the missing case is decoration.

- **Checksums verify a backup.** They verify that the bytes are the bytes that were written. They say nothing about whether the bytes restore into a database that starts, or a vault that unseals, or a set of stores that agree with each other. The offsite manifest check (200 random objects a week, [[offsite-weekly-copy-contract]]) is a necessary transport check; it has never been the reason we trusted the offsite copy. The 2025 vault restore from a disk was.

- **Restoring each store proves the platform restores.** The drill's finding 3: PostgreSQL, the vault and the Kubernetes objects each restored fine, to timestamps a few minutes apart, and `auth-svc` would not start because a key reference in one pointed at a version absent from another. Consistency across stores is the hard part, and the only way to see it is to bring the whole thing up and log in.

- **The vendor documentation is a runbook.** The batch version restore existed for two years in the documentation and was first run during the incident ([[restore-2025-11-documents-prefix-deleted]]). Every appliance feature we might need under stress has now been run once on staging by the on-call, from our runbook, without the vendor page open.

## Things that turned out to matter more than expected

- **Deletion rights.** Nothing in the backups changed after November 2025; what changed is that no key can delete from the data buckets any more. The best restore is the one you do not need, and it came from a policy change, not a backup change.

- **A number for the restore rate.** 1.2 GB of WAL per 10 minutes, 25 objects a second for version restore, 250 MB/s from the offsite disks. Three numbers, remeasured at every drill, and every "how long will it take" answer since has been within 20 %.

- **A single target timestamp.** Pick one, choose every snapshot as the latest before it, and accept the loss of the minutes in between. Restoring each store to "its latest" produces four stores from four moments.

- **Envelopes.** The paper keys ([[backup-encryption-and-key-custody]]) were correct, and one had a character that could be read two ways. Found by using them, not by looking at them.

## The five checks a backup line must pass

Applied to every row of the inventory at the quarterly review; a row that fails any check is in bold in `BACKUPS.md` until it passes.

1. **Restored in the last 12 months**, for real or in a drill, into something that was then used (queried, logged into, served a request). Not "listed", not "checksummed".

2. **Three copies named**, with the failure each one survives written next to it, and the third one physically elsewhere.

3. **A restore-rate number** from the last restore, so the time to recover can be stated without guessing.

4. **A consistency partner**: which other store must be restored to the same timestamp, and where that is written in the runbook.

5. **Its key opens**: the private key for the encryption of this line has been read from custody (vault or envelope) and used in the last 12 months.

`hf-warehouse-cold-prod` fails check 1 and has since 2024. It is bold, it is the November plan, and until then the honest statement is that the 31 TB of cold data have a copy on `stash-b` and a copy offsite, both of which we believe and neither of which we have restored.

## What we still argue about

- Whether the drill should be a surprise. The people who would do it on a real Monday are the people who plan it, so a surprise drill tests their calendars more than the backups. We plan them and pretend nothing.

- Whether 35 days of PITR is too much or too little. The February restore needed 3 minutes back; the argument for 35 is the monthly close. Nobody has needed more than 9 days. Unchanged.

- Whether to encrypt documents at the object level. Same answer as last year: not with presigned URLs, and presigned URLs are not going away.
