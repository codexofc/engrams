---
name: incident-2026-01-duplicate-pod-uploads
description: Jan 2026, 3 % of PODs were uploaded twice because the completion call timed out after the object was stored, fixed by making the complete endpoint idempotent on document id and app 4.7 reusing the pending row
type: project
status: active
verified: 2026-02-03
---

# Incident: duplicate POD uploads (HF-1455)

## Symptom

Support noticed on 2026-01-08 that many loads had two identical POD photos. A query on `documents` grouped by `(load_id, sha256)` gave 1 870 duplicates over December, about 3 % of PODs. Storage cost was irrelevant, but shippers who get PODs by webhook received two `document.available` events and one accounting integration created two delivery lines.

## Mechanism

Upload flow (see the API note on presigned uploads): create pending row, PUT to storage, call `complete`. The app's HTTP timeout on `complete` was 8 s. On a slow connection at the end of the day, the server did its `HEAD` plus SHA-256 check (downloads the file, 4 to 12 MB) and answered after 8 s. The app saw a timeout, and its retry logic restarted the whole flow: new pending row, new upload, new complete. The first `complete` had actually succeeded server-side.

So the root cause was on both sides: the server took too long to complete, and the app retried from the beginning instead of retrying the last step.

## Fixes

App 4.7 (released 2026-01-21):

- `DocumentUploader` persists the document id it received at step 1 in the `document_uploads` table with the file path and the current step. A retry resumes from the current step, never restarts.
- Timeout on `complete` raised to 30 s. It is not a user-facing wait, the upload runs in the background isolate.
- If `complete` returns 409 `document_already_attached`, treat as success.

API (HF-1456, deployed 2026-01-12, before the app):

- `POST /v2/documents/{id}/complete` is idempotent: a second call on an `AVAILABLE` row returns 200 with the same body. It used to return 409.
- The SHA-256 check is now done asynchronously by a Messenger handler for files over 2 MB, so `complete` answers in under 300 ms. A mismatch marks the row `CORRUPT` and notifies the driver to retake the photo, which has happened 4 times since.

## Cleanup

The 1 870 duplicates were merged by `app:documents:dedupe --dry-run` then for real: keep the earliest row, delete the other row and object, no webhook emitted. The accounting integration was warned before.

## What we kept from this

The upload state machine on the device is now the model for every multi-step network operation: persist the step, resume the step. See [[offline-sync-architecture]] for the mutation outbox which already worked this way, the upload path just had not been built on it. And see [[pod-photo-compression]], which reduced file sizes at the same time and made the timeout less likely in the first place.
