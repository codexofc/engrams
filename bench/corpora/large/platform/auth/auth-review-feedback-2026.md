---
name: auth-review-feedback-2026
description: What the six-month review of auth-svc (April 2026) concluded, what worked, what we would do differently, and the rules reviewers now apply to any change in the auth code
type: feedback
status: active
verified: 2026-04-24
---

# Six months of auth-svc: review notes

Held 2026-04-21 with the platform team, the support lead and one person from product. The service itself is described in [[auth-service-overview]].

## What worked

- **Extracting before the crisis.** The service was six weeks old when the stuffing wave hit ([[incident-2025-12-credential-stuffing]]). Adding four throttle counters to a 14 000-line service with one job took a night; doing it in the monolith's security layer would have taken a week and a release train.

- **Yaml-free.** Unlike notifications, auth has almost no policy in configuration files: the rules are code with tests, because an auth rule that someone can change in a yaml without a test is a rule nobody reviews. Nobody asked for the opposite.

- **The audit log as the product.** `auth:events --user` answers most support questions in ten seconds. It was the cheapest feature and it is the one the support team names first.

- **Constant-time responses, once measured.** The March finding ([[incident-2026-03-reset-timing-enumeration]]) was embarrassing and the fix was a day. The lesson is in the rules below.

## What we would do differently

- **Alert on shape, not rate, from day one.** The stuffing wave was detected by a CPU alert. `AuthDistinctIpsHigh` (distinct failing IPs) should have existed at launch; every auth service gets stuffed eventually.

- **Ship the trusted device with the MFA, not after.** Trust ([[device-trust-and-remember-me]]) shipped two weeks after the opt-in phase began, and the early adopters got the challenge at every login for those two weeks; several of them un-enrolled and had to be re-convinced.

- **Talk to carrier admins earlier.** The shared account cleanup ([[abuse-shared-dispatcher-accounts]]) and the MFA rollout both moved fastest when the carrier admin got a list and a deadline. We spent the first month talking to end users.

- **Decide the pepper question.** HF-4336 is still open. Either do it or close it.

## Rules for reviewing auth changes

Applied to every merge request touching `auth-svc` since April 2026, in addition to the usual review.

1. Any endpoint that must not reveal whether an account exists has a timing test in the CI, and the reviewer runs it locally with the change.

2. Any new counter, lock or block answers the question "can an attacker use this against a legitimate user" in the MR description. If the answer is yes, it is not a lock ([[account-lockout-policy]] has the reasoning).

3. Any new sensitive operation (money, credentials, MFA settings) is on the step-up list and requires `mfa_at` within 5 minutes, device trust or not. The list is in one place, `StepUpOperations`, and the reviewer checks the MR adds to it rather than checking `mfa_at` inline.

4. Any new `auth_events` type is added to the enum with its `details` keys, and the MR shows a sample line from `auth:events`.

5. No new secret material in PostgreSQL. TOTP secrets, HMAC keys, signing keys go to the vault with a reference in the table.

6. Any change to what support can do goes through the support lead, and any change to what an org admin can do goes through product. Auth does not decide who may act, it makes sure the actor is who they say.

7. A rollback plan that does not log everyone out, or an explicit statement that it will and a communication plan. The October migration logged everyone out once, on purpose, and that was the last time.

## Open questions

- Hardware keys for the 90 internal accounts: two people asked, nobody has budgeted the time.

- Push-based approval instead of TOTP if the driver app ever gets a dispatcher mode.

- Whether `inactive` at 90 days should become 180 for shipper users, who log in seasonally. Product says yes, security says the reactivation-by-reset path makes it harmless either way. Unresolved, low stakes.
