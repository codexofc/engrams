---
name: search-team-preferences
description: Preferences of the search pair: the index is derived and disposable, read the row not the event, weights in a file with cases, one experiment at a time
type: user
status: active
verified: 2026-07-30
---

Preferences of the two people who run haystack, written after five months in production and two incidents.

- **The index is derived and disposable.** Anything we cannot rebuild from PostgreSQL in 15 minutes does not belong in it. If a design needs the index to remember something PostgreSQL does not, the design is wrong ([[search-reindex-runbook]] is the proof: we rebuild often and without fear).

- **Read the current row, not the event.** An event says "something changed"; the row says what is true. The one exception is additive counters (`bid_count`), and only because the nightly comparison checks them ([[search-indexing-pipeline]]).

- **Filters decide, scores order.** No component of the score may hide a load, no filter may depend on a score. When someone asks for "a bit less of X in results", it is a filter or it is a weight, never both.

- **Weights live in a file, with cases.** `ranking/v2.yaml` and `tests/ranking/cases/`. A weight changed without a case updated is a change nobody can explain in three months ([[search-relevance-rules-v2]]).

- **One experiment at a time, two weeks, by org** ([[search-ab-testing-ranking]]). We do not have the volume for anything cleverer and we do not pretend to.

- **No fuzzy matching on proper nouns.** Synonyms, yes, maintained by hand ([[search-synonyms-city-names]]). Edit distance on city names produces confident nonsense.

- **Measure on a production copy** before touching shard count, routing, analysers or refresh interval. Thirty minutes of restore and load test; the alternative was three days ([[search-incident-2026-05-shard-hotspot]]).

- **Look at the max, not the mean**, for anything per node or per shard.

- **Latency is the API's latency.** We measure what the carrier gets, not what haystack reports. haystack's own numbers are for debugging.

- **Nothing sensitive in the index.** If a field could embarrass a shipper in a search result, it is not indexed and not stored ([[search-index-mapping-loads]] has the list). The search response is an allowlist.

- **The fallback must be boring.** When haystack is down, a plain PostgreSQL query with a banner. No clever cache, no stale-serve of the last results; carriers would rather see "degraded" than wonder.

- **Support first.** `indexed_at` on the load, `hfctl load reindex`, the lag on the dashboard: if support cannot tell in two minutes whether a load is in the index and why not, we failed.

- **Language**: notes in whichever language the author thinks in, mappings and field names in English, ranking case names in English so grep works.
