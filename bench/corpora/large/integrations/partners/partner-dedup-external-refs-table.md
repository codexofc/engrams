---
name: partner-dedup-external-refs-table
description: The partner_load_refs table (external_id, previous ids, sync_state, payload hash), inbound resolution order and the dedup rule for mirrored listings
type: project
status: active
verified: 2026-06-12
---

# `partner_load_refs`: one row per load per partner

The table behind `hfctl partner refs`. Created with the Cargolink integration, reshaped in HF-3085 and HF-3210.

## Schema

```sql
CREATE TABLE partner_load_refs (
  load_id uuid NOT NULL REFERENCES loads(id),
  partner text NOT NULL,                       -- 'cargolink' | 'fretzone'
  direction text NOT NULL,                     -- 'outbound' (our load, their listing) | 'inbound' (their listing, our shadow load)
  external_id text,                            -- their id for the listing
  previous_external_ids text[] NOT NULL DEFAULT '{}',
  external_created_at timestamptz,
  sync_state text NOT NULL,                    -- IN_SYNC | PENDING_PUSH | PUSH_FAILED | PARTNER_STALE | UNLINKED
  pushed_payload_hash text,
  last_pushed_at timestamptz,
  last_pulled_at timestamptz,
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (load_id, partner)
);
CREATE UNIQUE INDEX uq_partner_refs_external ON partner_load_refs (partner, external_id) WHERE external_id IS NOT NULL;
CREATE INDEX idx_partner_refs_state ON partner_load_refs (partner, sync_state) WHERE sync_state <> 'IN_SYNC';
CREATE INDEX idx_partner_refs_prev ON partner_load_refs USING gin (previous_external_ids);
```

One row per load and partner, whatever the direction. The unique index on `(partner, external_id)` is what makes an inbound callback unambiguous.

## Resolving inbound messages

A Cargolink callback or a Fretzone pull item carries their listing id. Resolution order in `PartnerRefResolver::byExternalId(partner, id)`:

1. `external_id = id`, the current listing.

2. `id = ANY(previous_external_ids)`, a listing that was re-listed ([[partner-cargolink-sync-push-feed]]); bids on the old id are still ours.

3. Not found: log `partner_ref_unknown`, answer 404 to Cargolink (they stop retrying on 404), ignore the Fretzone item. About 10 per week, all listings created manually by a shipper in the partner's own UI with our prefix by habit.

The resolver is the only place that reads these columns for inbound; the handlers get a `load_id` or nothing.

## Sync states

- `IN_SYNC`: the last push was acknowledged and the hash matches the current load.

- `PENDING_PUSH`: a change exists, not yet acknowledged. For Fretzone this can last up to 10 minutes by design ([[partner-fretzone-overview]]).

- `PUSH_FAILED`: the partner refused or was unreachable after retries; `last_error` says why. Shown to the shipper since HF-3112.

- `PARTNER_STALE`: the nightly comparison found the partner's copy differs from what we think we pushed (they changed it, or lost it). Fixed by re-push in the reconciliation, or investigated.

- `UNLINKED`: support ran `hfctl partner unlink`; we stop syncing this load to this partner and hide the inbound shadow load if any. Reversible by `resync`.

## Dedup of inbound listings (HF-3210)

An inbound Fretzone listing is compared before creating a shadow load:

1. Exact: same shipper org (via the partner account link the shipper set up), same `reference` after prefix stripping, same `pickup_date`. Match: no shadow load, a row `direction = 'inbound'`, `sync_state = 'UNLINKED'`, `last_error = 'duplicate_of_direct_load'`, so the reconciliation does not keep re-finding it.

2. Fuzzy (in test since June): same shipper org, same pickup and delivery postcodes, pickup date within one day, no reference on either side. Match: same as above plus a notification to the shipper ("we did not import your Fretzone listing X because it looks like load Y").

Weekly figures since the exact rule: about 90 inbound listings per week skipped as duplicates; 11 per week would be caught by the fuzzy rule, 2 of which were false positives in the test period (two genuinely different loads, same postcodes, same day). The fuzzy rule ships with the notification precisely for those.

## Retention

Rows stay as long as the load. When a load is archived (180 days after termination), the row moves with it to `partner_load_refs_archive`. `previous_external_ids` are never pruned; a load lives at most a few weeks on a board and has at most three listing ids.

## Support

`hfctl partner refs <load_id>` prints the row per partner. `hfctl partner resync` sets `PENDING_PUSH` and dispatches a sync message. `hfctl partner unlink` sets `UNLINKED`. Nothing else writes to this table outside the sync handlers and the reconciliation.
