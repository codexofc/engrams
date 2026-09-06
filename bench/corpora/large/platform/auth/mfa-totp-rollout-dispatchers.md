---
name: mfa-totp-rollout-dispatchers
description: TOTP made mandatory for the 41 000 dispatcher accounts between Jan and Mar 2026 (HF-4310), enrolment by grace period then hard wall, 94 % enrolled before the wall, support load peaked at 60 tickets a day, what we changed midway
type: project
status: active
verified: 2026-04-20
---

# Mandatory TOTP for dispatchers

## Why dispatchers first

A dispatcher account can accept a load, assign a driver, and change a bank account for payouts. The credential stuffing wave of December 2025 ([[incident-2025-12-credential-stuffing]]) reached 140 dispatcher accounts with reused passwords, and although no payout was diverted, the review made MFA for that population the first item. Shipper admins came second (required since May 2026), drivers are authenticated by their phone and device and are not in scope.

## Design

- TOTP only (RFC 6238, 30-second step, 6 digits, one step of tolerance either side). No SMS second factor: SMS is the driver's first factor, and we did not want a second factor that a SIM swap defeats. No hardware keys yet, the demand was two accounts.

- Enrolment at `/v1/me/mfa/totp/setup` returns the secret (once) and an `otpauth://` URI; `/confirm` requires one valid code before the method is active. The secret is stored in the vault at `mfa/totp/<user_id>`, the table row holds only the reference ([[auth-service-overview]] for the tables).

- Ten recovery codes generated at confirmation, see [[mfa-recovery-codes]].

- The challenge happens after password verification, before the session is created: `POST /v1/auth/login` answers `202 {"challenge": "mfa", "challenge_token": ...}` and the client calls `/v1/auth/mfa/verify` with the code. The challenge token is a 5-minute single-use JWT, not a session.

- A trusted device ([[device-trust-and-remember-me]]) skips the challenge for 30 days. Without that, mandatory MFA on a dispatcher who logs in 6 times a day would have been refused by the product team, rightly.

## Rollout (HF-4310)

| Phase | Dates | Rule | Enrolled at end of phase |
|---|---|---|---|
| opt-in | 2026-01-12 to 01-25 | banner in the dispatch tool | 8 % |
| grace | 01-26 to 02-22 | interstitial at login, skippable 3 times a day | 61 % |
| soft wall | 02-23 to 03-08 | interstitial skippable once a day, e-mail reminders | 87 % |
| hard wall | 03-09 | no login without enrolment | 94 % on the day, 99.2 % by 03-31 |

The remaining 0.8 % were accounts that had not logged in since December; they enrol when they come back or are deactivated at 90 days by the usual inactivity rule.

## What we changed midway

- The interstitial originally offered "skip" as a prominent button. Enrolment in the first week of the grace phase was 11 %. Swapping the visual weight (enrol primary, "not now" as a text link) doubled the daily enrolment rate. Obvious in hindsight.

- Carriers with 20 or more dispatcher accounts got a spreadsheet from support listing who had not enrolled, sent to the carrier admin. This was the single most effective measure: 30 % of the remaining population enrolled within 3 days of the spreadsheets going out. The carrier admin has authority over their dispatchers; we do not.

- The hard wall was planned for 03-02 and pushed a week because two large carriers asked for time to distribute company phones to dispatchers who refused to install an authenticator on personal phones. Legitimate, and the works council question in Germany was raised there too. Those carriers now buy cheap Android phones for the office; we had nothing to do with that but it is why the wall moved.

## Support load

Tickets tagged `mfa` per day: 5 during opt-in, 20 to 30 during grace, 60 at the hard wall day, back to 10 a week later, 3 a day since April. Categories at the wall day: lost phone before saving recovery codes (40 %), time drift on the phone (25 %, fixed by the one-step tolerance plus the "check your phone's automatic time" hint added on 03-10), authenticator app confusion (20 %), "I do not want this" (15 %, answered by pointing at the carrier admin).

The lost-phone case is handled by the reset procedure in the recovery codes note; support cannot disable MFA, only reset it after identity verification, and a reset revokes every session.

## Measured effect

Between 2026-03-09 and 2026-06-30, `auth_events` shows 31 000 successful password verifications on dispatcher accounts that then failed or abandoned the MFA challenge. Some are users mistyping; the ones from IPs that also failed passwords on other accounts (the stuffing signature) are 9 400. That is the number of account takeovers the second factor blocked in four months, on a population where the earlier wave had reached 140 accounts in three days without it.

Payout bank account changes on dispatcher-side accounts, which require a fresh MFA code regardless of device trust (step-up, `mfa_at` within 5 minutes): 1 200 in the period, zero disputed.

## What we did not do

- No SMS fallback for TOTP. Two large carriers asked. The answer is recovery codes and the carrier admin's ability to trigger a reset for their own dispatchers (with identity verification steps they perform, logged).

- No per-carrier exemption. One carrier with a fleet of 300 asked for an exemption "because we have our own VPN". A VPN is not authentication of the person.

- No push-based MFA ("approve this login on your phone"). Would be nicer, would need our own app on every dispatcher's phone, and the dispatch tool has no mobile app. Revisit if the driver app ever gets a dispatcher mode.
