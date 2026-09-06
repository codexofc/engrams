---
name: api-key-hashing-and-prefix
description: Since HF-2087 a key is hfk_ plus 6 char prefix plus 32 base62 chars, stored as SHA-256 with the prefix in clear; the plaintext column was dropped 2026-01
type: project
status: active
verified: 2026-02-20
---

# API key format and storage (HF-2087)

Before this ticket the `api_keys.secret` column held the key in clear, see [[api-keys-legacy-plaintext]] for what that looked like and why it was a problem. HF-2087 changed the format, the storage and the lookup.

## Format

```
hfk_<prefix 6 chars>_<secret 32 chars>
```

- `hfk_` marks it as a Halden Freight key so that secret scanners (ours and the customers') can recognise it with one pattern: `hfk_[0-9A-Za-z]{6}_[0-9A-Za-z]{32}`. We registered that pattern in our own gitleaks config (see the `security/common` conventions) the same week.

- The prefix is random base62, unique in the table (`UNIQUE (prefix)`), and it is the only part stored in clear. It is what support and the customer see in the UI and in emails.

- The secret is 32 chars from `random_bytes(24)` encoded base62. About 190 bits, more than enough.

Test keys used to have a `test_` marker; we dropped it because the staging environment is a separate database with separate keys anyway, and a marker invited people to be careless with keys that were, in fact, real.

## Storage

`api_keys.secret_hash = sha256(full_key)` as hex. Plain SHA-256, no salt, no bcrypt: the input has 190 bits of entropy, so a rainbow table is not a threat and bcrypt would only cost us 100 ms per API call. This was debated in review; the argument that settled it is that bcrypt protects low-entropy secrets (passwords), and this is not one.

Lookup: `SELECT ... FROM api_keys WHERE prefix = :prefix AND deleted_at IS NULL`, then `hash_equals($row->secretHash, hash('sha256', $key))`. Constant-time compare, one indexed query, no scan.

## Migration

Three deploys, because we could not ask 3 000 integrators to rotate on the same day.

1. 2025-12-02: add `prefix` and `secret_hash`, backfill both from the plaintext column, dual-read (hash first, fallback to plaintext for keys created before the deploy), new keys in the new format only.

2. 2026-01-06: fallback removed. Old-format keys still worked because their hash had been backfilled, they simply did not have the `hfk_` shape. 71 % of active keys were still old-format at that date.

3. 2026-01-20: `ALTER TABLE api_keys DROP COLUMN secret`. A backup of the column was NOT kept, on purpose, checked by two people, the point being that it no longer exists anywhere.

Old-format keys remain valid until their `expires_at` (365 days max since HF-2095, see [[api-keys-lifecycle]]), so by early 2027 every key will be `hfk_`.

## What the change caught

The week after deploy 1, the customer-visible prefix let the support team answer "which key is this" for the first time. That is how we found that one carrier had the same key configured in three different TMS instances, one of which belonged to a company they no longer worked with. The key was rotated; the story is one of the inputs of the 90 day unused rule.

## Support tooling

`bin/console apikey:lookup hfk_Ab3dE9` prints owner organization, label, scopes, expiry and last use for a prefix. It never prints the secret, because it cannot.
