---
name: document-storage-s3-presigned
description: PODs, CMRs and carrier documents live in an S3-compatible bucket per environment, uploaded by clients through presigned PUT URLs (15 min), read through presigned GET (5 min), never proxied by the API
type: project
status: active
verified: 2026-03-11
---

# Document storage

Bucket per environment on the object storage of the cluster (`hf-documents-prod`, `-staging`, `-dev`), S3 API, accessed with `AsyncAws\S3\S3Client`. Credentials come from the `api-object-storage` secret managed by the ops team.

## Upload flow

1. Client calls `POST /v2/loads/{id}/documents` with `{ "kind": "POD", "content_type": "image/jpeg", "size": 812331, "sha256": "..." }`.

2. API creates a `documents` row in state `PENDING` with a v7 UUID, and returns a presigned `PUT` URL valid 15 minutes, restricted to that content type and a `Content-Length` range (size ± 0 bytes, we require the exact size).

3. Client PUTs the bytes directly to storage.

4. Client calls `POST /v2/documents/{id}/complete`. The API does a `HEAD` on the object, checks size and (for files under 20 MB) downloads and checks the SHA-256, then moves the row to `AVAILABLE` and emits `document.available` to the outbox (see [[webhook-delivery-outbox]]).

Rows stuck in `PENDING` for more than 1 hour are deleted by `app:documents:purge-pending` along with the object if it exists.

Key layout: `<org_uuid>/<load_uuid>/<document_uuid>.<ext>`. The org prefix is what lets us delete everything of an organisation at offboarding with a single prefix listing.

## Why not proxy through the API

The first version streamed uploads through php-fpm. A 12 MB POD photo held a php-fpm worker for the duration of a mobile upload on a bad connection, sometimes 40 s. With 12 workers per pod and drivers uploading at the end of their shift, the API queued. Presigned URLs removed that entirely. See the mobile note on photo compression for the client side of the same problem.

## Download

`GET /v2/documents/{id}` returns JSON with a presigned `GET` URL valid 5 minutes and `Content-Disposition` set to the original file name. The web front opens it in a new tab, the driver app downloads it to its cache. Never a redirect: the mobile HTTP client followed redirects and re-sent the `Authorization` header to the storage host, which logged it. Found during the 2025 security review, fixed in HF-1042.

## Retention

- POD, CMR, delivery notes: 10 years (legal, transport documents).

- Carrier insurance and licence documents: until 2 years after expiry.

- Load photos that are not PODs: 24 months.

Retention is enforced by a lifecycle rule on the bucket per key prefix, plus the `documents.retain_until` column for the API side. A `documents` row is never deleted before its object.

## Virus scanning

None. Discussed in HF-1043, deferred. Files are served with `Content-Disposition: attachment` and never rendered inline, which is the mitigation we accepted.
