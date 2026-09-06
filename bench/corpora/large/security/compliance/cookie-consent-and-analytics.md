---
name: cookie-consent-and-analytics
description: The app sets only strictly necessary cookies; product analytics go to our ClickHouse after opt-in in users.analytics_consent (58 %), no third-party tags
type: project
status: active
verified: 2026-02-24
---

# Cookies and analytics consent (HF-2073)

## Position

The web app (`app.halden.example`) and the marketing site are treated differently.

**The app** is used by authenticated professionals. It sets:

- the session cookie (`hf_session`, `HttpOnly`, `Secure`, `SameSite=Lax`), strictly necessary;

- `hf_org` (the last selected organization), strictly necessary for the multi-organization users, no consent needed;

- `hf_density` and `hf_lang` UI preferences, which we classify as strictly necessary because they only make the interface work as the user asked. The DPO agreed, with the note that they carry no identifier.

Nothing else. No third-party tag, no advertising pixel, no session replay. There is no cookie banner in the app, because there is nothing to consent to at the cookie level.

**Product analytics** (which screen was used, which button clicked, see the platform's event naming notes) do not use cookies but do profile behaviour. They are sent to our own ClickHouse, never to a third party, and only after **opt-in**. The prompt is shown once after the third login ("Help us improve: allow anonymous usage statistics?") and the answer goes to `users.analytics_consent` (`granted`, `refused`, `null`) with `analytics_consent_at`. Changeable any time in `/settings/privacy`. The front reads the flag from the `/me` payload and does not even load the analytics module when it is not `granted`.

Opt-in rate in February 2026: 58 % granted, 27 % refused, 15 % not yet asked or dismissed. Dismissing is treated as not granted.

Events carry `user_id` (not pseudonymised at collection, because dispatch analytics need to join with organization size), and the warehouse pseudonymises after 13 months as per [[data-retention-matrix]]. Events of users who later refuse are deleted by a weekly job, not just stopped: refusing means "forget what you collected", which is stricter than the regulation requires and easier to explain.

**The marketing site** (`www.halden.example`) has a consent banner because it has one third-party thing: an embedded video player on two pages. The banner is a 40 line script of ours, no consent management vendor. Refusing keeps the video as a click-to-load placeholder. There is no analytics on the marketing site at all; the growth team uses server-side page counts from the access logs, aggregated per path and day, no cookies.

## Mobile app

No consent prompt for analytics in the driver app: it collects crash reports (necessary to run the service, documented) and no product analytics. This was a deliberate choice: drivers are not in a position to say no to their employer's tool, so we do not ask them to say yes to something optional.

## What we said no to

- A third-party product analytics SaaS with session replay, proposed by the product team in 2025. Replay on a dispatch screen would record customer names and prices of other customers' loads. Our own ClickHouse events are enough for funnels.

- A consent management platform. Our needs are one flag per user and one banner on two marketing pages.

- Consent by inactivity ("continued use means acceptance"). It was in the 2024 footer text and is gone.

## Documentation

`https://docs.halden.example/legal/cookies` lists the four cookies with purpose and lifetime, and the analytics description. The page is generated from `config/compliance/cookies.yaml`, and a test asserts that every cookie set by the app in the functional test suite is listed there. That test caught a `hf_ab` experiment cookie in January 2026 before release.
