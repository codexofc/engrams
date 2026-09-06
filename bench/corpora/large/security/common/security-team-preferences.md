---
name: security-team-preferences
description: Security people want boring controls over tools, one implementation per security behaviour, rotate first ask later, no blame, and no gate on every MR
type: user
status: active
verified: 2026-06-20
---

# What the security people want

Six rota members and seven champions, overlapping. No dedicated security team, which shapes everything below.

- **Boring beats clever.** A 72 h cooling period, a 48 h quarantine, a kept previous secret, an allow-list. Every control that held in the last year was one of these; none was a product we bought. When someone proposes a tool, the question is "what boring rule does it replace, and why is the rule not enough".

- **One implementation per security behaviour**, and an architecture test that keeps it that way. Two incidents came from a second copy of something already fixed once. `DependencyRulesTest` is the most valuable test in the API repository by incidents prevented per line.

- **Rotate first, ask later.** A secret seen where it should not be gets rotated before anyone discusses whether it "really" leaked. The cost of an unnecessary rotation is an hour; the cost of a necessary one delayed is an incident. Written in [[secrets-handling-conventions]], repeated here because it is the rule people hesitate on.

- **We are not a gate.** Champions review `security`-tagged MRs; the rest of the MRs merge without us. If we were required on every MR we would become the bottleneck people route around. The tag is a trust mechanism; abusing it (not tagging to avoid review) is the one thing that gets a direct conversation.

- **Findings are tickets with dates. Wishes are not findings.** A "we should probably" without a ticket does not exist. Pen-test findings, drill gaps, post-mortem actions: all tickets, all with due dates that do not move, all reviewed monthly.

- **No blame, and we mean the practice, not the slogan.** Post-mortems name roles. The person who clicked the phishing link is the person who reported it 33 minutes later, and that report is the best detection we had all year. Anyone who makes a reporter regret reporting will hear from us.

- **Say no with a reason, in writing.** Every note in these security projects has a "what we did not do" section. The next person who wants to reopen a decision should find the argument, not a wall.

- **Detection is half the work.** A control without an alert is a hope. The post-mortem template requires detection actions; the lite threat model ([[threat-model-lite-template]]) has a whole question for it.

- **Rota is a week, not a life.** 15 minute acknowledgement, nine paging alerts and not one more, written handover, no back-to-back weeks ([[security-oncall-rota]]). A rota that burns people out is a rota nobody volunteers for.

- **Read the incidents, not the frameworks.** The monthly champions session reads one real incident or finding. Two people did external certification courses; both came back saying the sessions taught more. We keep the sessions.

- **English for anything a customer may see** (post-mortem summaries, questionnaire answers, the TOM), French for internal runbooks and most tickets. Code and identifiers in English everywhere.

- **Measure the burden.** Rota hours per week, champion hours per month, pre-commit false positive rate ([[gitleaks-precommit-feedback]]). A control whose cost we cannot state is a control we cannot defend when someone wants to remove it.

What we do not want: a compliance-driven checklist of 200 controls nobody reads, security review as a separate phase after "the code is done", or being asked to approve something in a chat message at 18:00 on a Friday without a ticket.
