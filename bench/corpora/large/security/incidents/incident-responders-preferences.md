---
name: incident-responders-preferences
description: Responders want containment before diagnosis, a scribe who does nothing else, UTC everywhere, no private threads, evidence copied before cleanup
type: user
status: active
verified: 2026-06-15
---

# How the incident responders want it done

Collected from the debriefs of the six incidents and the March drill. The process note ([[incident-process-severity-levels]]) says what to do; this says how the people doing it prefer it, and why.

- **Contain first, understand later.** Revoke, isolate, rotate. If the containment turns out unnecessary, we lost twenty minutes of someone's convenience. The reverse costs hours of exposure. The commander who hesitated 15 minutes in an early 2025 incident to "confirm it was really a leak" is the story we tell new responders.

- **The scribe does nothing else.** Not "also looks at the logs". The timeline is the only artefact that survives the incident intact, and every other role gets pulled into fixing. If there are only two people, the second one is the scribe, and the commander fixes. Tooling in [[incident-timeline-tooling]].

- **UTC in the channel, in the timeline, in the post-mortem.** The team spans three time zones and daylight saving shifts twice a year. Local time appears only in parentheses when human behaviour explains something (first coffee, end of shift).

- **No private threads during an incident.** Everything in `#inc-HF-xxxx`. A direct message between two responders is invisible to the scribe and to whoever takes over at the shift change. If something is too sensitive for the channel (a person's name in a phishing case), the channel gets "discussed privately with the commander, decision: X".

- **Copy the evidence before touching anything.** Bucket access logs, pod logs, the malicious tarball, the phishing email with headers. The commander says explicitly "evidence secured, you may now clean up". Twice in 2025 a restart erased what we needed.

- **Say what you are about to do before doing it**, in the channel, when it is reversible-but-disruptive (rotating a shared key, freezing an account). Ten seconds of typing, and someone may know why it is a bad idea right now.

- **A page to the rota is answered in 15 minutes, by a human, in the channel.** "Ack, looking" is enough. Silence for 15 minutes means the backup gets paged.

- **Post-mortems name systems and roles.** "The reviewer" not a name; "the bucket module" not "X's module". This is not politeness, it is what makes people declare incidents ([[incidents-lessons-2025-2026]] has the phishing example).

- **Post-mortem within 5 working days, or a dated reason why not.** Memory decays; the two 2025 post-mortems written weeks later have `[memory]` on a third of their timeline lines.

- **Comms from a template, reviewed by the commander, never improvised** ([[incident-comms-templates]]). The one improvised customer message of 2025 said "no data was compromised" four hours before the logs were read.

- **Drills count.** Two a year, treated with the same seriousness as an incident. The people who found the drill "artificial" in March had changed their minds by the end of the debrief ([[security-incident-drill-2026-03]]).

- **Close the incident explicitly.** "Incident closed at HH:MM UTC, exposure ended at HH:MM, post-mortem owner: <role>." An incident that fades out is an incident whose actions get lost.

What responders do not want: a war-room video call open for hours (audio fatigue, nothing written down), management asking for ETAs in the incident channel (there is a separate `#inc-HF-xxxx-status` for that in SEV1), and being asked "how did this happen" before containment is declared.
