---
name: restore-2025-11-documents-prefix-deleted
description: Nov 2025: a wrong-prefix cleanup deleted 212 000 document versions in hf-documents-prod, all restored from versioning in 3 h 40, 0 lost, HF-4602
type: project
status: active
verified: 2025-12-09
---

# Restore 2025-11-26: documents prefix deleted

## What happened

A one-off cleanup of expired customer exports was written as `s3 rm --recursive s3://hf-exports-prod/2025-0` and run from `ops-tools` with the ops admin key. The person ran it first in a shell where the previous command had set an environment variable pointing the CLI's default bucket at `hf-documents-prod` for an unrelated check. The command as executed was `rm --recursive` on `hf-documents-prod/2025-0`, which is the prefix of every document uploaded in the first nine months of 2025 whose key starts with the upload year and month (`2025-01/...` to `2025-09/...`).

The CLI deleted 212 000 objects in 11 minutes before the person noticed the object count in the terminal was not the 3 000 they expected and hit Ctrl-C.

## Why nothing was lost

`hf-documents-prod` has versioning enabled ([[bucket-versioning-and-lifecycle-rules]]). A delete on a versioned bucket writes a delete marker; the previous version stays and is subject to the noncurrent-version lifecycle rule (90 days). So at 14:22 there were 212 000 delete markers and 212 000 intact noncurrent versions. From the API's point of view the documents were gone: `GET` on the key returned 404, and the presigned URLs the clients had would have too.

## Impact before the restore

14:11 to 17:50. Support tickets: 31, all "the document does not open" from carriers and shippers looking at loads from earlier in 2025. The OCR pipeline, which reads new documents only, was untouched. Two invoice disputes needed a POD from June that was unavailable for the afternoon; both were resolved the next day. The dispatch tool showed the document list from PostgreSQL (unchanged) with broken downloads.

## The restore

1. 14:25 the on-call listed delete markers created after 14:10 with the appliance's `list-object-versions` filtered on `IsLatest` and `DeleteMarker`, paged by 1 000: 212 400 markers. The list took 12 minutes and was saved to a file on `ops-tools`, with a copy in the ticket.

2. 14:40 test on 100 keys: delete the delete marker (`delete-object --version-id <marker id>`), which makes the previous version current again. `GET` works. Checksums of 10 restored objects against the `documents.sha256` column in PostgreSQL: match.

3. 14:50 the Stashbox appliance has a batch operation for exactly this (`stashctl restore-versions --bucket hf-documents-prod --manifest markers.csv`), which the vendor's documentation describes and nobody had used. Ran it on 1 000 keys: 40 s. Ran it on the rest.

4. 17:35 batch complete. A comparison job (`documents.key` in PostgreSQL versus `HEAD` on the bucket for every key uploaded in 2025-01 to 2025-09, 212 000 requests from 8 parallel workers, 15 minutes) found 0 missing.

5. 17:50 support told to close the tickets. The two disputes got their PODs.

Total 3 h 40 from deletion to verified restore, of which 2 h 45 was the batch and the verification.

## What changed (HF-4602)

- **No application key has `DeleteObject` on `hf-documents-prod` or `hf-pg-backups-prod`.** Deletion is lifecycle only. The documents API never deleted anything anyway (retention handled by lifecycle from `documents.retain_until`); the right was there because the policy had been copied from a template.

- **The ops admin key cannot delete in `hf-documents-prod` either.** A per-incident write key with deletion rights is issued by the storage on-call, expires in 4 hours, and the issuance is logged. Two admins can issue, the issuance needs a ticket. It has been issued twice since, both for lifecycle debugging.

- **The CLI on `ops-tools` has no default bucket** (the environment variable that made this possible is unset by the shell profile), and a wrapper prints the resolved bucket and prefix and asks for confirmation on any `rm --recursive` with more than 1 000 matches (it lists first, which costs seconds and has already stopped one more mistake in March 2026).

- **Delete markers are alerted**: more than 1 000 delete markers created in 10 minutes on any versioned bucket pages the storage on-call. The appliance emits the count; the alert would have fired at 14:12.

- **Versioning retention on `hf-documents-prod` went from 30 to 90 days**, on the reasoning that a deletion noticed by a customer three weeks later must still be recoverable.

- The batch restore command is in the runbook with the exact syntax, and the [[restore-drill-2026-05]] repeated it on a staging bucket to check that the on-call could do it without the vendor documentation open.

## Numbers

| | Value |
|---|---|
| objects deleted (delete markers) | 212 400 |
| objects lost | 0 |
| time to first restored object | 29 min |
| time to verified full restore | 3 h 40 |
| support tickets | 31 |
| batch restore throughput | about 25 objects/s, single appliance |

## Lesson

Versioning is the backup that restores in an afternoon; the offsite copy ([[backup-inventory-and-retention]]) would have restored the same objects in two days. Both exist for different failures, and this one was the cheap kind because versioning was on. The expensive part was human: 11 minutes of deletion before someone looked at the terminal. The wrapper and the alert are the fixes for that, and the removed delete rights are the fix for the next person.

## Commands, as they went into the runbook

```
# list delete markers created after a timestamp, paged
stashctl list-object-versions --bucket hf-documents-prod --prefix 2025-0 \
  --delete-markers-only --since 2025-11-26T14:10:00Z --output csv > markers.csv

# restore by removing the markers, in batches, with a report
stashctl restore-versions --bucket hf-documents-prod --manifest markers.csv \
  --batch-size 1000 --report restore-report.csv

# verify against the database
documents-verify --bucket hf-documents-prod --since 2025-01-01 --until 2025-09-30 --workers 8
```

`documents-verify` is the 80-line script written that afternoon (`HEAD` per key from `documents`, compare `Content-Length` and, for a 1 % sample, the checksum), kept in `halden-infra/storage/tools/`. The `--dry-run` flag on `restore-versions` prints what it would do, and the runbook says to run it first on 100 keys, which is what the on-call did in the drill.
