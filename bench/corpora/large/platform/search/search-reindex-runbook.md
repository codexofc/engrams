---
name: search-reindex-runbook
description: Full reindex into a new physical index: create, dual-write, 12-minute backfill, verify counts and shard balance, switch aliases, keep old 48 h, rollback
type: reference
status: active
verified: 2026-07-24
---

# Full reindex runbook

Needed when the mapping changes in a non-additive way ([[search-index-mapping-loads]]), when the analyser or synonyms change, or when the index is suspected corrupt. Done seven times since March 2026, about 40 minutes end to end, no downtime. The [[search-incident-2026-05-shard-hotspot]] note is the one time it went wrong; step 6 exists because of it.

## Preconditions

- The indexer version to deploy builds documents for the **new** mapping and still accepts the old alias layout.

- A ticket with the reason and the mapping diff.

- Not between 06:00 and 10:00 CET on a weekday. Not on a Friday afternoon.

- `haystack` green, indexer lag under 5 s.

## Steps

1. **Create the new index** from the mapping file, with the same settings:

```
curl -u haystack-indexer -XPUT https://haystack.hf.internal:9200/loads-v8 \
  -H 'content-type: application/json' -d @mappings/loads-v8.json
```

The file contains settings (6 primaries, 1 replica, refresh 5 s, `dynamic: strict`) and mappings. For the backfill, temporarily set `refresh_interval: -1` and `number_of_replicas: 0` on the new index; it indexes about twice as fast. Restore both before switching.

2. **Dual-write**: point `loads-write` at both indices in one atomic aliases call (`actions: add loads-v8 to loads-write`). From now on every live change lands in both. `loads-read` still points at `loads-v7`.

3. **Deploy the indexer** version that knows the new mapping. Watch `hf_haystack_indexer_rejects_total` for 5 minutes; a mapping mismatch shows here immediately.

4. **Backfill**: from a job pod,

```
haystack-indexer --mode=backfill --target loads-v8 --batch 2000 --parallel 4
```

Reads `loads` where `status IN ('OPEN','BIDDING')` or terminated within 7 days, in `id` order, from the read replica, and bulk-indexes with `external_gte` versions. Live writes arriving during the backfill for the same load are reconciled by the version: the higher wins. About 12 minutes for 90 000 documents; the job prints progress every 10 000. It is idempotent; if it dies, run it again.

5. **Restore settings** on `loads-v8`: `refresh_interval: 5s`, `number_of_replicas: 1`. Wait for green (replicas take 1 to 2 minutes).

6. **Verify**:

- `GET loads-v7/_count` and `GET loads-v8/_count` with `searchable = true`: within 0.1 % of each other (live writes explain small differences).

- `GET _cat/shards/loads-v8?v&s=docs`: the largest shard under 1.5 times the smallest. If not, stop; something about routing changed.

- Run the ranking test suite against `loads-v8` directly (`RANKING_INDEX=loads-v8 make ranking-test`); all 63 cases pass.

- Run five real searches from the shell script `scripts/search-smoke.sh loads-v8` and compare result counts with `loads-v7`.

7. **Switch read**: one atomic aliases call, `remove loads-v7 from loads-read, add loads-v8 to loads-read`. The search API picks it up on the next query. Watch p50 and p99 for 10 minutes ([[search-query-latency-slo]] dashboard).

8. **Stop dual-write**: `remove loads-v7 from loads-write`. The old index freezes at this point.

9. **Keep `loads-v7` 48 hours**, then delete it. During those 48 h a rollback is two aliases calls (step 10). The nightly comparison runs against `loads-read` and will check the new index that night.

## Rollback

If anything is wrong after step 7 and before step 9: `loads-read` back to `loads-v7` (one call). If step 8 was done, `loads-v7` is missing the writes since then; re-add it to `loads-write` first, then run the backfill against `loads-v7` for the last hour (`--since 1h`) to catch up, then switch read. Never happened for real; rehearsed on staging with every reindex.

## Snapshot restore instead of backfill

When PostgreSQL itself is the problem (a bad migration on `loads`, a replica with stale data), restore yesterday's snapshot ([[search-haystack-cluster-layout]]) as `loads-restore-<date>`, point `loads-read` to it, fix PostgreSQL, then do a normal reindex. Search shows yesterday's loads for the duration, with the API refusing bids on those that moved; better than nothing, worse than a working index. Tested quarterly, never used in anger.

## Log

Each reindex gets a line in `docs/reindex-log.md` in the indexer repo: date, from, to, reason, duration, anything odd. Seven lines so far.
