---
name: playbook-driver-cannot-log-in-old
description: Historical driver login playbook from before app 4.5 (no lockout, no SMS link, PIN reset by carrier admin), kept to read old tickets
type: reference
status: archived
superseded_by: [[playbook-driver-cannot-log-in]]
verified: 2025-10-02
---

# Driver cannot log in (playbook until app 4.5)

Kept for reading tickets from 2025. The current procedure is [[playbook-driver-cannot-log-in]].

## How it worked then

Drivers logged in with phone number and a 6-digit PIN set by the carrier in the back-office. There was no lockout counter on the server and no local counter in the app: a wrong PIN could be retried forever. There was no SMS link either. A forgotten PIN meant the carrier admin typed a new one in the back-office and told the driver by phone.

## Checks (old)

1. Find the driver by phone in the back-office (there was no `hfctl driver find` yet, `hfctl` 1.6 only had `org` and `load`).

2. Ask the carrier to reset the PIN in the back-office. Tell them the PIN cannot be `000000` or `123456`, the two values they kept trying.

3. If the driver had two phones, both stayed logged in. There was no single-device rule, so a driver leaving a company kept access until the carrier disabled the account. This was the source of the Kowalczyk case in November 2025 and the reason for the single-device rule in 4.5.

4. If nothing worked, ask for the app version and the phone model, and escalate.

## Why it was replaced

Three things at once in app 4.5 (November 2025): the local argon2id PIN check with a 5-attempt lockout, the one-device rule, and the SMS link to set a PIN without the carrier admin. The support flow changed completely: the lockout created a new failure mode (`pin_attempts = 5`) that needs `hfctl driver unlock`, and the PIN reset by phone disappeared.

Tickets from before November 2025 mention "PIN reset by admin" and "both phones work": that is this version.
