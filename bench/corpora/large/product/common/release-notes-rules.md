---
name: release-notes-rules
description: Release notes per audience from the ticket field, published Tuesdays, translated, never for a feature behind a flag under 100 %
type: reference
status: active
verified: 2026-01-20
---

## Sources

Every ticket that changes something a user can see has a `release_note` field with one to three sentences in English, written by the person who did the work, reviewed by the product manager of the area. A ticket without a release note cannot be moved to `done` if it carries the `user_visible` label. The publication tool assembles the notes from the tickets shipped since the last publication.

## Audiences

Four channels, one text each:

- **Shippers** (web app, in-app "what's new" panel and a monthly email).
- **Carriers** (web and mobile app, in-app panel).
- **Drivers** (mobile app store listing text and in-app panel, kept very short, drivers read it on a phone in a truck).
- **Internal** (back office, support and finance), which includes everything, plus configuration changes and flag rollouts.

A ticket's note is tagged with its audiences. The shipper and carrier notes are translated into FR, DE, PL, RO, NL by the localisation vendor within 48 hours; the internal notes stay in English.

## Rules

- Publication every Tuesday at 10:00 CET, whatever was shipped. No release notes on Friday deploys; they wait for Tuesday.
- A feature behind a flag ([[feature-flags-convention]]) is not announced until the flag is at 100 % for that audience. Announcing an experiment variant to everyone produced 30 support tickets in 2025 from users who did not have it.
- Write what changed for the user, not what was done in the code. "You can now photograph the certified copy of your licence" and not "Added copy_number field to LicenceExtractor".
- Bug fixes are listed only if a user could have noticed the bug. The line says what now works, not what was broken.
- No promises about upcoming work. A release note describes the past.
- Pricing or billing changes that affect money get a date and, if a rate changed, the old and new values. Finance reads these to answer shippers.

## Retention and where they live

Notes are stored in `release_notes` (`published_at`, `audience`, `locale`, `body`, `ticket_keys`) and rendered at `/whats-new` in each app. The in-app panel shows a dot until the user opens it once; 34 % of shippers and 18 % of carriers open it in the week of publication.

## Escalation

If a note is wrong after publication (it happened for a pricing rate that was misquoted in December 2025), the correction is a new entry dated the day of the correction, not an edit of the published one, with the support escalation path in [[support-escalation-path]] informed so that support has the right figure.
