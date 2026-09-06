---
name: incident-2026-03-dependency-typosquat
description: A typosquatted npm package reached a bot MR on 2026-03-17 and was caught in lockfile review; led to registry.hf.internal with a 48 h quarantine
type: project
status: active
verified: 2026-04-03
---

# Typosquatted dependency caught before merge (March 2026)

Post-mortem `2026-03-17-npm-typosquat.md`. **SEV3**: a control failed (a malicious package reached a merge request) and another caught it (review). Nothing was executed on a developer machine or in CI. Still written up in full because the margin was one reviewer's habit.

## What happened

- **2026-03-17 03:40 UTC** `[forge]`: the automated dependency update bot opened an MR on the web front repository. Among 14 bumps, one line in `package-lock.json` replaced `date-fns-tz@2.0.1` with a package whose name differed by one transposed letter, version `2.0.2`, published 5 days earlier. The `package.json` entry had not changed; the lockfile resolution had, because the bot's resolver followed a `peerDependencies` hint in another updated package that pointed at the look-alike name.

- **08:50** `[forge]`: a front engineer reviews the MR. Their habit: expand the lockfile diff and read every changed `resolved` URL. They notice the name and the fact that the tarball URL points at a scoped namespace unrelated to the original maintainer.

- **08:58** `[chat]`: posted in the security channel with the diff. Incident opened at 09:05, SEV3.

- **09:20**: the package tarball is downloaded to an isolated VM and read. `postinstall` script fetches a second-stage script from a remote host and runs it with the environment (which in CI would have included the masked-but-present forge token). Classic.

- **09:30**: MR closed, branch deleted. CI had **not** run `npm ci` on the MR because the bot's MRs run a lint-only pipeline until a human approves; that rule dates from a CI cost measure in 2025 and turned out to be a security control by accident.

- **10:00**: the registry is notified through its abuse form; the package was removed 2026-03-19.

- **10:30**: check whether any other repository resolved the same name: `grep` over every lockfile in the forge, none.

- **11:00**: incident closed.

## Root cause

Dependencies were resolved directly against the public registry, with no allow-list and no delay between a package's publication and its availability to us. A one-letter name variant was as legitimate to the tooling as the real thing.

## What went well

- A reviewer who reads lockfile diffs. The post-mortem names this as a habit worth institutionalising, not as an individual heroic act.

- Bot MRs not installing dependencies before approval.

## What went badly

- Nothing automated looked at the name, the age, or the publisher of a newly resolved package.

- The web front's `package.json` allowed `^` ranges everywhere, so the resolver had freedom it did not need.

## Actions

- **Registry proxy with quarantine** (HF-2138, done 2026-04-01): all package managers (npm, Composer, pub) resolve through `registry.hf.internal`, which proxies the public registries and refuses any package version **published less than 48 hours ago** unless explicitly allowed. The delay is the cheapest defence against a freshly published malicious version; most are removed within a day.

- **Name similarity check**: the proxy compares a newly requested package name against the names already present in any of our lockfiles (Damerau-Levenshtein distance 1) and blocks with a message naming the existing package. Two false positives in the first two weeks, both resolved in a minute with an allow-list entry.

- **Lockfile diff summary in MRs**: a CI step posts a comment listing every changed `resolved` URL with the package's age and publisher, so what the reviewer did by habit is now in front of every reviewer.

- **Exact versions** in `package.json` for the web front (`save-exact=true` in `.npmrc`). Ranges stay allowed in libraries, not in deployables.

- Documented in the shared security conventions under dependency updates; this note is the incident, that note is the policy.

Ticket: HF-2136.
