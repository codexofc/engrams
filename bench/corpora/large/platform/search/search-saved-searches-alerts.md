---
name: search-saved-searches-alerts
description: Saved searches and push alerts (HF-3175): percolator index matches each new load, one push per user per 10 minutes, figures after two months
type: project
status: active
verified: 2026-08-14
---

# Saved searches and "new load" alerts (HF-3175)

Carriers asked for it in every survey: "tell me when a load appears on my lane". Shipped in app 4.9 and the web in June 2026.

## Saving a search

Any search (point, radius, delivery zone, dates as a relative range, vehicle types, ADR, gabarit, free text) can be saved with a name. Stored in PostgreSQL `carrier_saved_searches` (`id`, `carrier_org_id`, `user_id`, `name`, `query jsonb`, `alerts_enabled`, `created_at`). Up to 20 per user. Dates are stored relative ("next 7 days"), resolved at query time, so a saved search does not go stale.

The saved searches also feed the `fit` component of the ranking ([[search-relevance-rules-v2]]): a load matching one of the carrier's saved searches scores higher in any search they run.

## Matching new loads

The naive approach (run every saved search every minute) is 20 000 saved searches times 60 runs an hour. Instead we invert it with a **percolator**: the saved search's query is indexed as a document in a separate index `saved-searches-v1` (field `query` of type `percolate`, mapped against the loads mapping). When the indexer indexes a load that just became searchable (a `load.published` or a return to `OPEN`), it also sends the document to `saved-searches-v1/_search` with a `percolate` query, which returns the saved searches whose query matches that document. One query per new load, about 2 000 per day, 15 ms each.

The percolator sees the same filters as the live search (radius, dates, vehicle types) except the `PRIVATE` visibility, which is checked afterwards in PostgreSQL (favourites can change). The ranking's score is not involved; a match is a match.

## Sending

Matches go to the `search-alerts` Messenger transport. `SavedSearchAlertHandler`:

- groups by `user_id`, waits 10 minutes from the first match (a Redis key per user), then sends **one** push notification: "3 new loads match 'Lyon to Milan'". The 10-minute window was chosen against the Monday-morning import pattern (1 200 loads in 4 minutes would have been 1 200 pushes for a carrier with a wide search).

- respects the user's quiet hours (default 21:00 to 06:00 local, configurable), queuing until morning.

- at most 12 pushes per user per day; beyond that, a single "many new loads today" at the end.

- the push opens the app on the saved search's results, sorted by the usual ranking.

E-mail digest as an alternative for users who turned push off: one e-mail at 07:00 with the night's matches.

## Figures, June 15 to August 14

- 14 200 saved searches by 6 900 users (about 40 % of active carrier users). Median 1 per user, max 20 (a dispatcher at a large carrier, one per lane).

- 61 % have alerts enabled.

- 380 000 matches, 92 000 pushes sent after grouping (4 matches per push on average), 2 100 held by quiet hours per night.

- Push open rate 34 %. Bids within 30 minutes of a push: 11 % of opens. Compared with the same carriers' bid rate on organic searches (6 %), alerted loads convert almost twice as often, which is the number the product team wanted.

- Time from `load.published` to push, p50 6 min (the grouping window dominates), p95 11 min.

## Problems seen

- A carrier with a 800 km radius and no vehicle filter matched everything; 12 pushes a day, all "many new loads". They asked us to "fix the spam". The app now warns when saving a search that would have matched more than 200 loads in the last 7 days.

- Percolator index and loads mapping must stay in step: the July mapping change added a field the percolator mapping did not have, and every saved search using it (none yet, luckily) would have failed to match. The reindex runbook now includes updating `saved-searches-v1` mapping in the same step.

- Deleted saved searches must be removed from the percolator index too; a bug in June left 300 orphans matching and sending nothing (the handler found no row). Harmless, fixed, and the nightly comparison now covers this index.

## Not done

No alerts for shippers ("a carrier bid on your load" already exists through the normal notifications). No alert on price changes of already-seen loads; requested twice, not enough.

## Operations

The percolator index `saved-searches-v1` has 1 primary, 1 replica (14 000 small documents). It is rebuilt from PostgreSQL by `haystack-indexer --mode=percolator-backfill` in under a minute; `carrier_saved_searches` is the source of truth, the percolator index is a cache like the loads index. The nightly comparison checks the two document counts and re-emits missing or orphan queries.

Metrics: `hf_search_alert_matches_total`, `hf_search_alert_pushes_total{channel}`, `hf_search_alert_delay_seconds` (publish to push). Alerts: `SearchAlertsStalled` if no push in 30 minutes between 07:00 and 20:00 on a weekday (notify), `SearchAlertsPercolateErrors` above 1 % (page, it means the percolator mapping and the loads mapping diverged).

The push itself goes through the mobile team's notification service; a failure there is theirs, and shows as `hf_search_alert_pushes_total{result="failed"}`.

## Support

`hfctl carrier saved-searches <carrier_org_id>` lists the saved searches with their last match time and the number of pushes in 7 days. The two questions support gets: "I did not get an alert for load X" (check the load's visibility and the saved search's filters, then `hfctl search percolate <load_id> --org <carrier_org_id>`, which replays the percolation and prints which saved searches match and why not), and "I get too many" (the 200-load warning and the quiet hours).
