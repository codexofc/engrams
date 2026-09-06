---
name: partner-cargolink-overview
description: Cargolink, German load board since 2025-10: loads out, bids in, push feed and callbacks, volumes, contract terms that shape the code, owners
type: reference
status: active
verified: 2026-07-16
---

# Cargolink integration overview

Cargolink is a fictional load board operating mainly in Germany, Austria and Poland, with about 9 000 active carriers, many of whom are not on Halden. We integrate with them since October 2025 (HF-3015) so that our shippers' loads reach their carriers and their carriers' bids reach our shippers. This is the map of the integration; details are in the linked notes.

## Directions

**Loads: Halden to Cargolink.** A load with `visibility = PUBLIC` or `PARTNER_ONLY` from a shipper whose org has the Cargolink partner enabled is pushed to Cargolink as a listing. Updates (price, dates, cancel) are pushed too. Cargolink never creates loads on our side from their own shippers' listings: the flow is one-directional for loads, unlike Fretzone ([[partner-fretzone-overview]]). Since [[partner-cargolink-sync-push-feed]] (March 2026) the push is event-driven, under a minute; before that it was polling ([[partner-cargolink-sync-polling-v1]]).

**Bids and acceptances: Cargolink to Halden.** A Cargolink carrier bids on our listing on their site; Cargolink calls our callback `POST /v2/partners/cargolink/callbacks` with the bid. We create a bid on the load attributed to a shadow carrier org ([[partner-bid-relay-and-shadow-carriers]]). The shipper accepts on our side (or on theirs, Cargolink pushes the acceptance the same way); the load moves to `DISPATCHED`. Execution (driver app, tracking, POD) happens on Halden: the Cargolink carrier gets a driver account on our side, created at acceptance, with the login SMS.

**Invoicing.** On Halden. Cargolink takes a commission on loads dispatched to their carriers, invoiced by them to us monthly; our reconciliation job produces the figure we expect ([[partner-reconciliation-nightly-job]]).

## Endpoints and auth

Theirs, called by us (`https://api.cargolink.example/v3/`): `listings` (create, update, delete), `listings/{id}/status`, `carriers/{id}`. Auth: OAuth2 client credentials, token cached 50 minutes (they issue 60), client id and secret in the secret store under `partners/cargolink/*`, never in env vars.

Ours, called by them: `POST /v2/partners/cargolink/callbacks` for `bid.placed`, `bid.withdrawn`, `acceptance`, `listing.rejected`. Auth: HMAC signature on their side, our verification mirrors what we ask integrators to do. Their signing secret is rotated quarterly by them with a 48-hour overlap; we hold two.

## Volumes (June 2026)

- Listings pushed: about 2 300 per week, 60 % from six large shippers.

- Bids received: about 4 100 per week, 1.8 per listing on average; 38 % of our pushed listings get at least one Cargolink bid.

- Loads dispatched to Cargolink carriers: about 520 per week, 11 % of our total dispatched volume. This is why the integration matters commercially.

- Callback failures on our side: under 0.1 %, all 4xx on their malformed retries after their own incident in May.

## Contract terms that shape the code

Not the whole contract, only what the code enforces.

- **Commission** of a fixed percentage on the accepted bid amount for loads dispatched to a Cargolink carrier; zero if the same carrier was already active on Halden in the previous 90 days (the "existing relationship" clause). The shadow-carrier matching ([[partner-bid-relay-and-shadow-carriers]]) exists partly to compute this.

- **No exclusivity** either way. A load can be on Cargolink, Fretzone and Halden at once.

- **Price floor per lane**: Cargolink refuses listings under their minimum per km per corridor (`price_below_minimum`). We surface the error to the shipper since HF-3112.

- **Listing lifetime**: 14 days maximum on their side. A load still `OPEN` after 14 days must be re-pushed as a new listing; the sync does this and keeps the mapping in `partner_load_refs` ([[partner-dedup-external-refs-table]]). This clause caused the December 2025 incident ([[partner-incident-2025-12-cargolink-mass-expiry]]).

- **Data**: we may show Cargolink carriers' names and ratings to our shippers; we may not export them. Shadow carrier orgs are excluded from the data platform's carrier dimension.

## Owners

Us: the integrations team (three people), the partner contact is the team lead. Them: a technical account manager and an API team reachable through a shared ticket queue, response within one business day in practice. Their status page is `status.cargolink.example`; our sync alerts link to it.

## Environments

Their sandbox is described in [[partner-sandbox-environments]]. Staging pushes to their sandbox; production to production. There is no shared test data; we create listings in their sandbox with a `HFTEST-` reference prefix that their side filters from their carriers' view.

## What Cargolink does not do

No tracking feed to them (they asked; our shippers own the tracking data and we did not want to build a second consumer). No document exchange (POD stays on Halden; their carrier sees it in our app). No invoicing on their side.

## Things that surprised us

- Their listing ids are not unique across time: a deleted listing's id can be reused months later for another customer's listing. Our resolver only searches our own rows, so it does not matter, but a naive join in the warehouse on their id alone produced nonsense once.

- Their sandbox filters `HFTEST-` but their production did not filter anything, so a load test accidentally run against production in November 2025 showed 300 fake listings to real carriers for 20 minutes. The production credentials have been readable by the deploy role only since that afternoon.

- They count a listing "modified" whenever the payload hash changes, including field order. Our mapper sorts keys before serialising since v1's phantom updates.

- Their carriers can bid on an expired listing for 7 days after expiry. This is the December incident's mechanism and is still true; the re-list with `previous_listing_id` is what makes those bids land.

- Their commission statement arrives as a spreadsheet by e-mail on the 3rd. Our reconciliation exports the same format so finance compares two sheets. A CSV would be better; they said no.

## Where to look when it breaks

Grafana "Partners", partner filter `cargolink`: sync delay, push results, callback rate and callback errors, quota counter. Alerts route to `#integrations` and page only for `PartnerSyncBacklog` and callback 5xx above 1 %. Their status page is the first thing to open, then `hfctl partner runs --partner cargolink --since 1h`.
