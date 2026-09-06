---
name: support-hfctl-cli-cheatsheet
description: The hfctl commands support actually uses, grouped by object (org, load, driver, invoice, webhook, partner), with the flags that matter, the roles they need and the ones that are dry-run by default
type: reference
status: active
verified: 2026-08-14
---

# hfctl cheatsheet for support

`hfctl` is the support CLI ([[support-tooling-backoffice-console]]). It talks to `admin-api.hf.internal`, logs every call to `sys_audit_log`, and refuses anything your SSO role does not allow. Version pinned by the team: `hfctl 1.14`, check with `hfctl version`. Output is a table by default, `-o json` for piping.

Every mutating command is **dry-run by default** and prints what it would do. Add `--apply` to do it. This is the single most important thing to remember and the one people forget when a customer is on the phone.

## Orgs and users

```
hfctl org get <org_id>                     # plan, status, KYC state, feature flags
hfctl org users <org_id>                   # users, roles, last login, SSO or password
hfctl org flags <org_id>                   # feature flags resolved for this org
hfctl user unlock <user_id> --apply        # shipper web user locked after 10 bad passwords
hfctl user sso-reset <user_id> --apply     # forces re-link of SSO identity (L2)
```

`org get` shows `kyc_state` from Verifid as we store it (`PENDING`, `VERIFIED`, `REJECTED`, `EXPIRED`). It is a copy, refreshed by the KYC worker, and can lag Verifid by up to 15 minutes.

## Loads

```
hfctl load get <load_id>                   # status, carrier, driver, windows, partner refs
hfctl load events <load_id>                # load_events timeline with actors
hfctl load deliveries <load_id>            # webhook deliveries emitted for this load
hfctl load cancel <load_id> --reason <code> --apply       # cancel from DRAFT/OPEN/BIDDING/DISPATCHED (L2)
hfctl load cancel-in-transit <load_id> --reason <code> --ticket HF-xxxx --apply   # L2 + second approver
hfctl load undeliver <load_id> --ticket HF-xxxx --apply   # revert a wrong DELIVERED (L2, see playbook)
hfctl load reindex <load_id> --apply       # push the load to the haystack indexer again
```

`--reason` takes a code from `hfctl load cancel-reasons`. Free text goes in the Deskline ticket, not in the reason.

`cancel-in-transit` requires `--ticket` and prints a confirmation token that a second L2 or the backend on-call must enter within 10 minutes. Two people, always.

## Drivers

```
hfctl driver get <driver_id>               # phone (masked), carrier, devices, PIN attempts, last sync
hfctl driver find --phone +33612345678     # by E.164 phone, exact match
hfctl driver unlock <driver_id> --apply    # resets the bad-PIN counter server side
hfctl driver pin-link <driver_id> --apply  # sends the one-time SMS link to set a new PIN
hfctl driver devices <driver_id>           # device list with app version and last seen
hfctl driver revoke-device <driver_id> <device_id> --apply
hfctl driver sync-status <driver_id>       # outbox depth as last reported by the app, last push
```

`pin-link` is rate limited to 3 per driver per day. The fourth call is refused with `pin_link_rate_limited`, wait for tomorrow or escalate.

## Invoices

```
hfctl invoice get <invoice_id>
hfctl invoice resend <invoice_id> --to <email> --apply     # resends the PDF e-mail
hfctl invoice suppression-check <email>    # is this address on the bounce suppression list
hfctl invoice suppression-remove <email> --apply
```

No command edits an invoice. Disputes go through the credit note flow owned by billing.

## Webhooks

```
hfctl webhooks subs <org_id>               # subscriptions, status, disabled_reason
hfctl webhooks deliveries <sub_id> --status DEAD --since 7d
hfctl webhooks replay <delivery_id> --apply
hfctl webhooks replay-range <sub_id> --from <ts> --to <ts> --apply   # L2, max 24 h range
hfctl webhooks enable <sub_id> --apply     # after too_many_failures
hfctl webhooks test <sub_id> --apply       # sends a ping event
```

## Partners

```
hfctl partner refs <load_id>               # external refs on Cargolink / Fretzone
hfctl partner resync <load_id> --partner cargolink --apply
hfctl partner runs --partner fretzone --since 24h
```

## Paging and misc

```
hfctl page backend --reason "<one line>" --ticket <deskline id>
hfctl audit <user_login> --since 24h       # what did a colleague do
hfctl whoami
```

`page backend` asks for confirmation and reminds you of the criteria from [[support-escalation-path]]. It does not check them, you do.

## Habits

Copy the command you ran into the Deskline internal note, with its output. `hfctl` output includes the audit id, which is what the backend asks for when they need to find your action.
