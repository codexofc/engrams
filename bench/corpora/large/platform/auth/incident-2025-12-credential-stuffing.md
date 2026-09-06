---
name: incident-2025-12-credential-stuffing
description: Dec 2025: 2.1 M login attempts from 9 000 IPs over 3 days, 140 accounts entered with reused passwords, no money moved, throttle and MFA followed, HF-4320
type: project
status: active
verified: 2026-01-28
---

# Incident 2025-12-08 to 12-11: credential stuffing

## What happened

Starting 2025-12-08 around 22:00 UTC, `POST /v1/auth/login` received a stream of attempts with valid-looking e-mail and password pairs, from about 9 000 source IPs (residential proxies, mostly, with a long tail of hosting ranges), spread over three days. 2.1 M attempts in total, peaking at 38 a second on the night of the 9th. The pairs came from some public breach corpus; the e-mails were 63 % unknown to us, 37 % matched accounts.

Of the matching accounts, 140 were entered: the password was the same as in the leak. 112 shipper users, 28 dispatchers. The attackers logged in, listed loads and, on 19 accounts, opened the payout settings page. No bank account was changed: the payout change required re-entering the password at the time (it now requires a fresh TOTP code) and the attackers did not bother. No load was created, accepted or cancelled by them. We think it was reconnaissance for resale of the accounts.

## Detection

Too slow. `auth-svc` was six weeks old ([[auth-service-overview]]) and had an alert on the login failure rate (`warn` above 20 %), which fired at 22:40 on the 8th and was read the next morning as "someone forgot their password a lot". The `page` came from an unrelated place: at 02:10 on the 9th the Argon2id load put the three `auth-svc` pods at their CPU limit and `POST /v1/auth/login` p99 crossed 5 s, which paged the platform on-call through the generic latency alert.

The on-call scaled `auth-svc` to 8 replicas at 02:20 (which helped the attackers as much as us), looked at the source IPs, and by 03:00 understood what it was.

## Response, in order

1. 2025-12-09 03:10: per-IP limit lowered from 100 to 20 attempts an hour, and a per-e-mail limit of 10 attempts an hour introduced in a hurry, both in the ingress rate limiter first, then moved into `LoginThrottle` in the day ([[login-rate-limiting-rules]] has the rules as they stand now).

2. 04:00: the 140 entered accounts identified from `auth_events` (`login.success` from an IP that also had 50 or more `login.failure` on other accounts in the hour). All 2 300 sessions of those users revoked with reason `stuffing_response` ([[session-model-and-revocation]]), passwords marked `breached_at`, forced change at next login, `auth.security_alert` e-mail sent.

3. 2025-12-09 morning: support briefed with a canned answer; 60 tickets over the week, none about actual damage.

4. 2025-12-10: the whole leaked e-mail list that matched our users (14 000 accounts) checked against the breach list at next login; 3 100 more had a breached password (not necessarily the leaked one) and got the 7-day banner from [[password-policy-and-argon2]].

5. 2025-12-11: attempts stopped by themselves around noon. The last IPs were hitting 20 an hour, which made the exercise pointless for them.

## Why the limits did not exist

They existed in the monolith as a Symfony login throttling of 5 attempts per minute per e-mail and IP pair. The pair rule is useless against stuffing: every attempt is a new pair. The extraction copied the rule as it was. The review asked the obvious question and the answer was that nobody had thought about the attack shape.

## What came out of it (HF-4320 and children)

- `LoginThrottle` with four independent counters (per IP, per e-mail, per IP and e-mail, and a global failure rate), applied before the hash. Documented in the rate limiting note.

- The failure rate alert became a `page` at 40 % over 10 minutes with at least 1 000 attempts, and a second alert `AuthDistinctIpsHigh` pages when more than 500 distinct IPs fail in 10 minutes, which is the stuffing signature and does not depend on the rate.

- The mandatory TOTP programme for dispatchers ([[mfa-totp-rollout-dispatchers]]) got its date. Shipper admins followed in May 2026.

- Payout bank account changes require a fresh TOTP code, not the password. Password re-entry proves nothing when the password is what leaked.

- A weekly job compares new breach corpora against `users.email` and marks accounts for the breach banner without waiting for them to log in.

- The reset flow's token life went from 24 h to 30 min while we were at it, and the reset request endpoint got its constant-time treatment (which then turned out to need measuring, see [[incident-2026-03-reset-timing-enumeration]]).

## Numbers

| | Value |
|---|---|
| attempts | 2 110 000 |
| distinct source IPs | 9 100 |
| distinct e-mails tried | 610 000 |
| e-mails matching an account | 226 000 (37 %) |
| accounts entered | 140 |
| sessions revoked | 2 300 |
| payout pages opened by attackers | 19 |
| bank accounts changed | 0 |
| support tickets | 60 |
| extra compute for the three days | 8 replicas instead of 3, about 90 EUR |

## What we did not do

- Block by country. The IPs were everywhere, including where our customers are.

- Add a CAPTCHA to the login. Discussed, refused for now: our dispatchers log in from shared workstations six times a day and the friction is real, while the per-IP and per-e-mail limits made the attack uneconomic by themselves. If a wave beats the limits with more IPs, a CAPTCHA after the third failure per e-mail is the drafted next step (HF-4325, not started).

- Notify the 226 000 matched users whose password was not the leaked one. Their account was not entered; telling them "your e-mail is in a leak somewhere" is true of everyone.
