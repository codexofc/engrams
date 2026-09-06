---
name: dependency-updates-policy
description: Weekly bot MRs resolve via registry.hf.internal (48 h quarantine, name similarity block), advisories triaged in 2 working days, exact pins, monthly SBOM diff
type: reference
status: active
verified: 2026-06-08
---

# Dependency update policy

Applies to every deployable repository (API, web front, mobile app, data jobs, EDI and telematics gateways) and to the shared libraries. The incident that shaped half of it is the typosquat of March 2026 in the incidents project.

## Sources

Every package manager resolves through **`registry.hf.internal`**, a caching proxy in front of the public npm, Composer, pub and PyPI registries. No direct registry access from CI or from developer machines on the office network (blocked at the egress; from home, the tooling config points at the proxy and a direct install works but leaves a trace in the lockfile that the MR check rejects).

The proxy enforces:

- **48 hour quarantine**: a version published less than 48 h ago is refused with a message naming the version and when it becomes available. Override per package and version through `allowlist.yaml` in `halden-security`, MR required, two approvals, used 9 times in 3 months, mostly for our own libraries.

- **Name similarity block**: a package name at edit distance 1 from any name in any of our lockfiles is refused unless allow-listed. The block message names the existing package.

- **Yanked and advisory awareness**: a version with a known advisory of severity high or critical is refused unless allow-listed with a reason and an expiry date.

## Cadence

- **Weekly bot MRs**, Monday 04:00 UTC, one MR per repository grouping minor and patch updates, one MR per major update. Bot MRs run lint and a lockfile diff summary only; the full pipeline runs after a human approves. The summary comment lists every changed `resolved` URL with publisher and package age.

- **Reviewer duty**: the repository's rota (not the security rota) reviews and merges within the week. An unmerged bot MR older than 14 days shows up in the monthly review.

- **Major updates** are scheduled work with a ticket, not merged from the bot MR.

## Security advisories

The proxy and the forge's advisory feed both raise an issue in the `security-advisories` YouTrack board when a dependency we use gets an advisory.

- **Critical or high**: triage within **2 working days** by the security rota ([[security-oncall-rota]]): exploitable in our usage or not, and if yes, patch or mitigate within 7 days. "Not exploitable" is a written sentence with the reason, and the ticket stays open until the patched version is in anyway (at most one cycle).

- **Medium and low**: next weekly MR.

Numbers for H1 2026: 41 advisories, 6 high, 1 exploitable in our usage (a PDF library used for invoice rendering, patched in 3 days), median triage time 1 day.

## Pinning

- Deployables pin **exact versions** (`save-exact` for npm, `composer.lock` committed, `pubspec.lock` committed, `requirements.txt` with hashes for the data jobs).

- Libraries we publish internally use ranges, as libraries should.

- Docker base images are pinned by digest, updated by the same weekly bot.

## SBOM

Each release pipeline produces an SBOM (CycloneDX JSON) stored with the release artefact. A monthly job diffs the current production SBOMs against the previous month and posts the diff (new packages, removed packages, new publishers) to the security channel. The rota reads it; the review is cheap and it is the only place where "we now depend on 14 more packages than last month" becomes visible.

## What we do not do

- Auto-merge bot MRs, even for patch versions. The lockfile diff review is the control that caught the typosquat; automating the merge removes it.

- Vendor dependencies into the repository. Proposed after the typosquat, rejected: it hides updates instead of controlling them.

- Block all packages younger than a week. 48 h catches most removed-within-a-day malicious versions; a week would have blocked a Symfony security release we needed.

The secure coding checklist ([[secure-coding-checklist-php]]) refers here for the "new dependency" item, and the threat model template ([[threat-model-lite-template]]) asks which third-party packages a feature adds.
