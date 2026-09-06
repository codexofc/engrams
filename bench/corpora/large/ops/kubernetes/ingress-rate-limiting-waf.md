---
name: ingress-rate-limiting-waf
description: Edge protection is layered, ingress-nginx per-IP limits on auth routes (20 rps), a ModSecurity CRS at paranoia level 1 in detection-only on the API and blocking on the web front, and the connection limit that stopped the Feb 2026 credential stuffing
type: project
status: active
verified: 2026-03-08
---

# Rate limiting and WAF at the ingress (HF-INFRA-401)

The application has its own per-organisation limiter (documented in the API project). This note is about what happens before a request reaches PHP.

## Per-IP limits (ingress-nginx annotations)

On the API ingress, only for the unauthenticated routes, because for authenticated ones the application knows the organisation and the ingress does not:

- `/v2/auth/login`, `/v2/auth/driver-login`, `/v2/auth/token`: `limit-rps: 20`, `limit-burst-multiplier: 3`, `limit-connections: 50`. Exceeding gives a 503 from nginx (not 429, that is how ingress-nginx does it, and we did not add a custom error page for the API because the clients treat 503 as "retry later" already).

- `/.well-known/jwks.json` and `/health`: `limit-rps: 50`.

- Everything else on the API: no ingress limit.

On the web front ingress: `limit-rps: 100` per IP on `/` (static assets are cached by the browser, a real user never approaches that), `limit-connections: 100`.

The limiter's memory zone is per controller replica, so with two ingress nodes the effective limit is up to double when a client is spread across both by the VIP. Accepted.

## The February 2026 credential stuffing

On 2026-02-11 from 22:40, about 900 IPs (residential proxies) tried e-mail and password pairs on `/v2/auth/login` at roughly 4 requests per second per IP, 3 600 per second in total. The application limiter (10 attempts per 15 minutes per identifier and per IP) held, no account was accessed (the pairs were from an unrelated leak), but 3 600 rps of PHP doing argon2 verification put the API pods at 100 % CPU and the HPA scaled to 60, the maximum. Real users saw p99 at 2 s for 25 minutes.

What stopped it: `limit-connections: 50` and `limit-rps: 20` on the login routes were **not** in place at the time. Added at 23:05 by editing the overlay and syncing, and the load dropped to nothing in 3 minutes, since each IP was over 20 rps. The attack moved on after another 20 minutes.

Follow-up decisions:

- The per-IP limits above, permanent.

- Argon2 parameters on the API side stay (65 MB, 3 iterations), the cost is the point. But the login handler now checks the per-identifier limiter **before** hashing, which it did after, so a rejected attempt costs nothing.

- A `WafBlockedRequestsSpike` alert (`warn`) and a `LoginRateHigh` alert (`page` above 200 logins per second cluster-wide) so the next one is seen in minutes.

## ModSecurity with the OWASP Core Rule Set

Enabled in the ingress-nginx controller (`enable-modsecurity: true`, `enable-owasp-modsecurity-crs: true`), paranoia level 1, anomaly threshold 5.

- **Web front ingress**: `SecRuleEngine On` (blocking). The front only serves static files and the risk of a false positive is nil.

- **API ingress**: `SecRuleEngine DetectionOnly`. The API receives JSON bodies with free text (goods descriptions, messages between dispatchers and carriers) and level 1 CRS flags a description containing `SELECT` or `<script>` written by a human. In two months of detection-only, 140 matches per day, 138 of them false positives on `/v2/messages`. Blocking would have broken messaging. We keep it in detection for the audit log value and revisit if a real attack pattern shows up.

ModSecurity costs about 8 % CPU on the ingress controller and adds 1 ms of latency at p50. Measured before enabling.

## What is not at the ingress

- Geo blocking: refused by product, we have carriers everywhere.

- Bot detection by fingerprinting: not needed so far.

- The application limiter and the outbox relay egress rules are covered elsewhere, see [[network-policies-baseline]] and [[ingress-nginx-config]].

## CRS tuning notes

What it took to make detection-only usable on the API and blocking usable on the front, in `components/ingress/modsecurity/`:

- Paranoia level 1 with `SecAction "id:900000,phase:1,pass,nolog,setvar:tx.paranoia_level=1"`. Level 2 was tried for a week on the front: 40 false positives a day on the file names of uploaded documents (rule 920440, file extension check, triggered by `.cmr` and `.pod` extensions we use internally). Back to 1.

- Anomaly threshold 5 for inbound (`tx.inbound_anomaly_score_threshold=5`), outbound checks disabled entirely (`SecResponseBodyAccess Off`), since we do not want the WAF reading JSON responses with load data.

- Rules removed by id for the API host only: 942100 (SQL injection library check, which flagged carrier names containing an apostrophe), 941100 and 941110 (XSS on any `<` in a message body), 920420 (content type check, because the mobile app sends `application/json; charset=utf-8` with a space that an old CRS version disliked). Each removal is a `SecRuleRemoveById` line with a comment holding the ticket and an example of the false positive.

- Request body inspection limited to 128 KB (`SecRequestBodyLimit 131072`) with `SecRequestBodyLimitAction ProcessPartial`, because the largest legitimate JSON body (a load creation with 20 stops) is 40 KB and anything larger is an export request that goes elsewhere.

- The audit log goes to stdout in JSON (`SecAuditLogFormat JSON`, `SecAuditLogParts ABHZ`, no request body part) and reaches Loki with `source="ingress"` and the `modsec` field set, retention 14 days. The `WafBlockedRequestsSpike` alert reads `increase(nginx_ingress_controller_modsecurity_blocked_total[5m]) > 50`, a metric we add ourselves from the audit log with a small Loki recording rule, since the controller does not expose one.

Measured over March 2026 on the front (blocking): 210 blocked requests, all from scanners hitting `/wp-admin`, `/.env` and the like, zero legitimate users blocked, zero support tickets. On the API (detection): 4 200 matches, of which 3 messages containing `' OR 1=1` typed by a dispatcher as a joke, and the rest false positives on free text. The decision to keep the API in detection stands, and the review date for it is written in the component README as September 2026.
