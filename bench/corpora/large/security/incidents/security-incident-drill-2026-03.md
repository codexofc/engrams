---
name: security-incident-drill-2026-03
description: The March 2026 tabletop (stolen support laptop) took 2 h 10 and found 5 gaps, from authority portal credentials to who calls the customer; twice-yearly now
type: feedback
status: active
verified: 2026-04-10
---

# Tabletop drill, March 2026

First structured exercise, run 2026-03-19 after the public bucket incident ([[incident-2026-02-pod-bucket-public]]) showed that the notification track was improvised. Seven participants: security rota (2), head of engineering, support lead, one platform engineer, the DPO (remote), and a facilitator from the ops team who had prepared the scenario and injected events.

## Scenario

"A support agent's laptop was stolen from a café at 12:30 local with the lid open, back-office and support console logged in, an impersonation session on a carrier account started 10 minutes earlier." Injects every 15 to 20 minutes: the agent calls at 13:05; the device management console shows the laptop online at 13:40 from an unknown network; a customer emails at 14:20 asking why their account shows a support session they did not request.

Not simulated: nothing was actually done to production. Every action was declared ("I would now run X") and the facilitator answered with what X would have shown, from prepared notes.

## What worked

- Declaration and roles: `/incident sécurité` within 3 minutes of the agent's call, commander and scribe named, SEV1 chosen correctly (staff credentials and an active impersonation session out of our control).

- Containment sequence was known: revoke the agent's Idento sessions, revoke the impersonation session (the IAM project's mechanism), rotate the agent's password and MFA, disable the device in the management console. Declared within 12 minutes of declaration.

- The impersonation guardrails were understood: read-only by default with a write allow-list, 30 minute lifetime, so the worst case on the carrier account was bounded and known.

## Gaps found

1. **Authority portal credentials.** When the DPO said "this may need notification, who files it", nobody knew where the portal login lived. Fixed the next day: two named holders, credentials in the vault, procedure line in the compliance project's 72 h note.

2. **Impact estimate.** "Which customer accounts did this agent impersonate in the last 30 days, and what did they see" took the platform engineer 25 minutes of query writing on the whiteboard. `compliance:breach:estimate` and `iam:agent-activity-report <staff_id> --days 30` now exist and answer in under a minute.

3. **DPO not in the channel** until someone thought of it at minute 35. The `/incident` command now adds the DPO account to every SEV1 channel automatically.

4. **Remote wipe was assumed, not verified.** The device management console can wipe, but nobody had ever confirmed a wipe actually completed on a stolen device (as opposed to a test device on the office network). Action: a quarterly wipe test on a spare laptop from outside the network. First one done 2026-04-02: wipe confirmed 6 minutes after the device came online.

5. **Who calls the customer.** The support lead and the commercial owner of the account both assumed the other would. The comms template ([[incident-comms-templates]]) now names the role: the support lead calls, the commercial owner is in copy.

## Metrics

- Time to declare: 3 min. Time to containment declared: 15 min. Time to first customer contact (simulated): 1 h 50, judged too slow; target 1 h for a SEV1 with a known affected customer.

- Duration: 2 h 10 including 30 minutes of debrief.

## Decisions

- Two drills a year, March and September, alternating a technical scenario and a people scenario. September 2026: a compromised CI runner.

- Each drill produces tickets the same way an incident does; the five gaps above are HF-2144 to HF-2148, all closed by 2026-04-10.

- The scenario notes and the facilitator's inject sheet are kept in `halden-security/drills/2026-03/` so the next facilitator does not start from nothing.

## Reactions

The support lead's comment in the debrief: "I learned more in two hours than in the incident process document". The DPO's: "the 72 hours felt very short once the clock was real". Both are the reason the cadence is twice a year and not once.
