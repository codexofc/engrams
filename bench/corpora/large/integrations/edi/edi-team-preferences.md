---
name: edi-team-preferences
description: The EDI pair adapts on our side rather than wait 6 weeks for a partner, keeps partner data in tables, uses real anonymised messages as fixtures, no DB access
type: user
status: active
verified: 2026-05-20
---

# How the EDI pair likes to work

Two people, five EDIFACT partners, two JSON shippers, a decade-old standard and TMSs older than that. Habits that came from tickets.

- **Adapt on our side first.** A partner change takes 6 weeks median and needs their release train. Our change takes a day. When a partner does something odd that we can absorb with a normalisation or an override, we absorb it and tell them anyway. We ask them to change only when the oddity is unsafe (the re-send convention in [[incident-2026-03-edi-duplicate-loads]]) or when it costs us on every message.

- **Partner specifics are data, never code.** Five tables, an override enum, an audit trail, a nightly export to Git for reading the diff ([[edi-mapping-tables-location]]). A reviewer who sees `if ($partner === 'nordkarton')` sends the MR back with a link to that note.

- **Every override has a ticket in its row.** The row says why it exists. Without that, nobody dares remove an override, and in three years we would have a hundred.

- **Fixtures are real messages, anonymised.** Sixty-one IFTMIN in the mapper test, all from production with names, phones and references replaced by a deterministic script. Synthetic messages test the standard; real ones test the partners. When a reject teaches us something, the message joins the fixtures.

- **A reject must be readable by the partner's EDI person without calling us.** Twelve reason codes, each with one English sentence that names the field and the value ([[edi-rejects-handling]]). "Invalid message" is not a reject reason.

- **Warnings over rejects when the load can exist.** A missing weight gets a default and a warning; a missing site code gets a reject because we cannot invent a place. The line is "can a carrier execute this load as it stands".

- **The month-end report goes out before the partner asks** ([[edi-reconciliation-daily]]). Fifteen disputes a month to two, for one PDF. Best return on effort in the project.

- **Push over pull, sync over async, when the partner can.** The JSON path ([[edi-api-json-alternative]]) exists because a synchronous 422 at integration time beats an APERAK a week into production.

- **We refuse direct database access** for partners and their integrators, every time, politely ([[edi-partner-nordkarton-quirks]] has the story). And we refuse "just enter these 30 loads by hand this once" when a partner's EDI is down: the reconciliation would show them as unexpected, the references would not match, and the next month-end would be a mess. Their web users can enter loads in the web app like any shipper; that is the fallback, and it reconciles.

- **Tickets in English when they involve a partner**, because we forward them; French otherwise. Reject reason sentences in English only, it is the language of every partner's EDI team.

- **The standard is a suggestion.** D.96A says what a segment means; each partner's implementation guide says what they do; the messages say what actually happens. We read all three and trust them in reverse order.

What tires us: being asked whether we "support EDIFACT" as a yes/no question. We support five partners' dialects and can add a sixth in six to ten weeks ([[edi-partners-overview]] has the durations). The syntax is the easy part.
