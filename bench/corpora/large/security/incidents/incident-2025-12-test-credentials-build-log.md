---
name: incident-2025-12-test-credentials-build-log
description: From 2025-11-30 to 12-09 the mobile CI job printed a staging password and a Payla sandbox secret in 214 build logs; led to masking and log scanning
type: project
status: active
verified: 2026-01-12
---

# Test credentials in a build log (December 2025)

Post-mortem `2025-12-09-ci-log-secrets.md`. Severity SEV2 under the current scale (internal exposure, no evidence of misuse). Handled under [[incident-process-v1]], one of the three incidents that motivated the new process.

## Summary

A shell step added to the mobile app's CI pipeline on 2025-11-30 ran `env | sort` to debug a build variable. The job's environment included `STAGING_API_PASSWORD` (the password of the staging test shipper account used by the end-to-end tests) and `PAYLA_SANDBOX_SECRET` (the sandbox API secret of our payment provider). Both were printed in clear in the job log of every pipeline run for 9 days, 214 runs. Job logs on the forge are readable by every member of the `mobile` group and of the `platform` parent group: about 60 accounts, all employees or contractors under NDA. Nothing indicates the values were used by anyone.

## Timeline (UTC)

- **2025-11-30 14:12** `[forge]`: MR merged adding `- run: env | sort` to `.ci/mobile-build.yaml` under a step named "debug flavour vars". The MR description says "temporary, will remove". Reviewed and approved by one person.

- **2025-11-30 14:20** `[forge]`: first pipeline run with the step. Log contains both secrets.

- **2025-12-09 10:05** `[chat]`: a platform engineer reading a failed mobile pipeline log to help a colleague notices `PAYLA_SANDBOX_SECRET=...` and posts in the security channel.

- **10:12** `[chat]`: incident opened. Commander: the platform engineer.

- **10:25** `[forge]`: the step is removed on `main` through a direct commit (bypassing review, agreed in the channel, later re-reviewed).

- **10:40** `[vault]`: `PAYLA_SANDBOX_SECRET` rotated in the payment provider's sandbox console and in the vault. Sandbox only; production secrets were never in that job's environment.

- **10:55** `[audit]`: `STAGING_API_PASSWORD` rotated; staging test account's sessions revoked.

- **11:30** `[forge]`: the 214 job logs deleted through the forge API. The forge keeps deleted logs 7 days in a trash we cannot purge ourselves; support request filed, confirmed purged 2025-12-12.

- **12:00**: staging `audit_events` searched for logins of the test account from non-CI IPs between 11-30 and 12-09: none. Payla sandbox activity log reviewed: only our own CI runs.

- **2025-12-09 13:00**: incident closed.

## Root cause

CI jobs received every secret of their environment as plain environment variables, and nothing between the job and the log looked for secrets. Printing the environment is a normal debugging move; the system made it dangerous.

Contributing: the two secrets were injected into the mobile build job at all. The build step needs neither; only the end-to-end test step does, and they ran in the same job.

## What went well

Someone read a log for an unrelated reason and recognised a secret. That is luck, not detection, and the post-mortem says so.

## Actions

- **Secret masking in CI** (HF-2081, done 2025-12-11): every variable marked `masked` in the forge is replaced by `[MASKED]` in logs. All vault-sourced variables are now marked. Masking is best-effort (it misses encodings), so it is the first layer, not the only one.

- **Secrets scanner on job output** (HF-2082, done 2025-12-19): a post-job step runs our gitleaks configuration (the one from the shared security conventions, patterns for our own key formats and vendor secrets) over the job log; a hit fails the pipeline and pages the security rota. First real catch: 2026-02, a base64 encoded service account key echoed by a misconfigured deploy script. Masking had not caught it, the scanner did.

- **Split the job** (HF-2084): build and end-to-end test are separate jobs; only the test job receives test credentials, and its log is restricted to the `mobile` group.

- **Review checklist**: "does this step print or upload anything that could contain a variable" added to the CI change checklist.

- **Time-boxed debug steps**: the step said "temporary". Nine days later it was still there. The rule now is that a debug step in CI comes with a ticket to remove it and a date, or it does not merge.

## Lesson we keep repeating

The word "sandbox" or "staging" in a secret's name does not make it harmless. The Payla sandbox secret could create test payouts that look real in our own staging invoicing, and the staging test account had `shipper_admin` on a staging organization that mirrors a real customer's data shape. Treat every secret as production until proven otherwise; see [[incidents-lessons-2025-2026]].

## How the scan on CI output works

Since it is the control that turned out to matter, the mechanics:

- A final `after_script` step in every pipeline template downloads the job's own log through the forge API (the job token has read access to its own log) and runs `gitleaks detect --no-git --source <log> --config gitleaks.toml`.

- A hit writes the finding (rule id, line number, masked value) to the job summary, exits non-zero so the job is red, and posts to the security channel with a link. The log itself is then deleted through the API by the same step, before anyone else sees it; the finding message contains enough to investigate (rule and line), not the value.

- Cost: about 4 seconds per job. Over 1 100 jobs a day, that is an hour of runner time per day, accepted.

- Known blind spot: a secret printed across two lines (some tools wrap at 80 columns) is missed. We tested and documented it; the mitigation is the masking layer, which does not care about line breaks.

Between December 2025 and June 2026 the scanner fired 9 times: 1 real (the base64 service account key), 8 false positives on test fixtures with IBAN-shaped strings, all resolved with allow-list entries in the first month.
