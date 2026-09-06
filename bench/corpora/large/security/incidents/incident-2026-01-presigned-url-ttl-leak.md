---
name: incident-2026-01-presigned-url-ttl-leak
description: In January 2026 POD links on the mobile-triggered email path were still valid 7 days after the pen-test fix; 41 000 emails; fixed by one DocumentUrlSigner
type: project
status: active
verified: 2026-02-06
---

# Presigned URLs still valid 7 days on one email path (January 2026)

Post-mortem `2026-01-21-presigned-ttl.md`. Severity SEV2 under the scale introduced weeks later: links were only in the recipients' mailboxes, no evidence of misuse, but a control we believed in place was not.

## Background

Pen-test finding M5 of October 2025 (compliance project) said POD document links in notification emails were presigned URLs valid **7 days**, and that forwarded emails therefore gave access to signatures and names for a week. The fix, HF-2062 (2025-11-12), reduced the TTL to 1 h and replaced the direct link by a link to the app, which re-signs on click. Marked closed after retest on a web-triggered "load delivered" email.

## What happened

- **2026-01-20 15:30 UTC** `[chat]`: a shipper's IT team, doing their own review, reports that a POD link in an email received on 2026-01-14 still opens. Forwarded to security.

- **15:45**: incident opened. Check: the email was the "POD uploaded" notification, which is triggered by the **mobile** upload path (`PodUploadedNotificationHandler`), not the web "load delivered" path. That handler built its own presigned URL with `S3PresignedUrlFactory::create($key, ttl: 604800)`, a copy of the pre-HF-2062 code. The pen-test fix had touched `DeliveryNotificationHandler` only.

- **16:10**: count: 41 200 "POD uploaded" emails sent since 2025-11-12 with 7 day links. Of those, links still valid at 16:10: about 5 600 (sent in the last 7 days).

- **16:30**: containment. There is no way to revoke a presigned URL other than rotating the signing credentials. The storage access key used for presigning was rotated at 16:40; every outstanding presigned URL (both paths, all TTLs) became invalid at once. Side effect: 1 h of broken links for everyone, including legitimate 1 h links, accepted and announced.

- **17:00**: fix deployed: both handlers use `DocumentUrlSigner::appLink($document)`, which returns an app URL, never a presigned URL. Presigned URLs are now generated only at click time by the API, TTL 15 min, for an authenticated user (the compliance project's retention note describes the same rule after the bucket incident).

- **2026-01-21**: bucket access logs for the 7 day window reviewed: `GetObject` on `pod/` keys via presigned URLs from IPs not matching the recipient organizations' known ranges: 0 identifiable. Access logs do not include the referrer, so "no evidence" is the honest conclusion.

## Root cause

Presigned URL generation was duplicated in two places, and the fix for the pen-test finding was verified against the path the consultant had tested, not against every place that produced the same kind of link. The system had two implementations of one security-relevant behaviour.

## What went well

A customer's review found it and told us within a day. The rotation of the signing key was a blunt but complete containment and took 10 minutes.

## What went badly

- The pen-test closure rule ("a test that reproduces the finding") was satisfied by a test on one path. The rule now reads "a test that reproduces the finding **on every code path that produces the same artefact**", and the reviewer of a finding closure asks "where else".

- We told the shipper on 2026-01-14 in a questionnaire answer that links expired after 1 h. The answer bank entry was corrected and the customer told.

## Actions

- `DocumentUrlSigner` is the only class allowed to call the presigner; an architecture test (`DependencyRulesTest`) fails if any other class references `S3PresignedUrlFactory`. Done 2026-01-22.

- Every email template that links to a document is listed in `config/notifications/document_links.yaml`, and a test renders each with a fixture and asserts the link points at `app.halden.example` and contains no `X-Amz-Signature`. Done 2026-01-28.

- A monthly scheduled check greps the last 30 days of sent email bodies (we keep rendered bodies 30 days for support) for `X-Amz-` and pages on a hit. Done 2026-02-02.

Ticket: HF-2103. Lessons carried into [[incidents-lessons-2025-2026]].
