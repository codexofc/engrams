---
name: login-bruteforce-protection
description: Login is throttled per account (10 failures, 15 min lock) and per IP (100/h); a credential-stuffing detector caught 41 000 attempts in one night in 2026-03
type: project
status: active
verified: 2026-04-16
---

# Brute force and credential stuffing on customer login

Applies to `POST /v2/auth/login` (web and mobile email login) and the driver PIN login. Staff login goes through Idento, which has its own brute-force detection, not covered here.

## Throttles (HF-2054, since 2025-11)

- **Per account**: 10 consecutive failures lock the account for 15 min. The counter is in Redis under `login:fail:{user_id}`, reset on success. The locked user gets an email "10 failed login attempts, your account is paused for 15 minutes; if this was not you, change your password". Response to the caller during lock is the same generic 401 `invalid_credentials` as a wrong password, so an attacker cannot tell a locked account from a wrong guess.

- **Per IP**: 100 failures per rolling hour, then 429 with `Retry-After` for the rest of the hour. IPs come from the trusted proxy header. Known corporate NAT ranges of large customers can be raised to 1 000 through `login_ip_allowances`, done for 4 shippers whose 300 dispatchers share two egress IPs.

- **Per PIN device**: a driver device gets 5 PIN failures then a 30 s, 60 s, 5 min back-off, enforced on the device and on the server. After 20 failures the device binding is removed and the driver must re-activate by SMS.

Failures are `auth.login_failed` in the audit trail with `details.reason` (`bad_password`, `locked`, `unknown_email`, `disabled`), see [[audit-trail-schema]].

## Credential stuffing (HF-2149)

Throttles per account and per IP do not stop a distributed list attack: many IPs, one attempt per account, thousands of accounts. On the night of 2026-03-09 we saw exactly that: 41 200 login failures in 6 hours from 3 900 IPs, 1 attempt per email for 38 000 distinct emails, of which 26 000 did not exist in our database. `unknown_email` failures went from a baseline of about 40 per hour to 4 000.

The detector added after that night is simple: `CredentialStuffingDetector` runs every 5 minutes and compares the count of `unknown_email` failures in the last 15 min against the 7 day median for that time slot. Ratio above 10 for two consecutive windows triggers:

1. A page to the security rota.

2. Login switches to "challenge mode" for 6 h: every login from an IP not seen for that account in the last 90 days requires a one-time email code. Legitimate users on their usual device are unaffected. `login_known_ips (user_id, ip_prefix /24, last_seen)` holds the history, 90 day TTL.

3. A report of accounts where the attempt **succeeded** during the attack window.

That night, 212 logins succeeded from attack IPs before the detector existed. All 212 accounts were force-reset the next morning: `security_stamp` rotated (see [[session-revocation-on-role-change]]), password reset email sent, and a support message explaining that a password they used elsewhere had probably leaked. Nine of them replied asking how we knew. No fraudulent action was found in the audit trail for any of the 212; the attacker seemed to be validating credentials, not using them.

## Passwords

Since the same ticket, new passwords are checked against a local copy of a breached-password hash list (k-anonymity lookup, first 5 hex chars of SHA-1 sent to an internal service `pwcheck.hf.internal`, nothing leaves the network). 3.1 % of new passwords in April 2026 were rejected by it.

Password rules otherwise: minimum 10 characters, no composition rules, no expiry.

## What we did not do

- CAPTCHA on login. The mobile app has no web view, dispatchers hated the idea, and the challenge mode covers the attack we actually see.

- Blocking by country. Our carriers are in 22 countries and the attack IPs were residential proxies in the same countries.
