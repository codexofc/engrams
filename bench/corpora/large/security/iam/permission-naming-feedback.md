---
name: permission-naming-feedback
description: Permissions named resource.verb and kept under 45 per audience made the custom role editor usable; noun-style names confused 6 of 8 admins in testing
type: feedback
status: active
verified: 2026-03-25
---

# What we learned naming permissions for customers

When custom roles shipped (HF-2141, see [[roles-permissions-model]]) shipper admins saw permission names for the first time. Until then the strings were internal. The usability sessions in February 2026 (8 admins from 6 organizations, 45 min each, remote) changed a few things.

## Findings

1. **`resource.action` with a plain verb reads fine.** `load.cancel`, `bid.accept`, `invoice.read` were understood by all 8 without explanation. The two we had named as nouns, `load.visibility` and `member.management`, were misread by 6 of 8 ("visibility of what? can I see loads or can I make them visible?"). Renamed to `load.read` and `member.invite` + `member.remove`.

2. **Count matters more than wording.** The shipper audience had 52 permissions. Admins scrolled, lost their place, and two of them gave up and picked a system role instead. We merged the fine-grained `invoice.read`, `invoice.download_pdf`, `invoice.list` into `invoice.read` (nobody had ever wanted them separately, the split was a leftover of the PDF rendering ticket) and did the same for four other groups. Down to 41. Under 45 is the working rule now.

3. **Group by resource, not by "level".** Our first layout grouped "basic / advanced / dangerous". Admins wanted "everything about loads together". Rebuilt the editor with one collapsible block per resource, dangerous ones (`load.cancel`, `member.remove`, `apikey.create`) with a red marker inside their block rather than in a separate section.

4. **Say what a permission implies.** `bid.accept` implicitly needs `load.read` (you cannot accept a bid on a load you cannot see). Rather than auto-adding dependencies silently, the editor shows "also requires: load.read" and ticks it, greyed out. `Permission::implies()` holds the graph, 9 edges in March 2026. Adding an edge requires a test that the voter grants the implied permission.

5. **Labels are translated, codes are not.** The code `load.cancel` is shown in monospace next to the label "Annuler un chargement" / "Cancel a load". Two admins said they liked seeing the code because it matched what their API integrator was reading in our documentation. Keep both.

## What we did not change

- Codes stay in English even in the French UI. Translating codes would break `role_permissions` fixtures and every integrator's mental model.

- No free-text description per permission in the editor. We wrote them, then removed them: nobody read a paragraph under a checkbox and it doubled the page height.

## Rules going forward

- A new permission is `resource.verb`, verb in the imperative, resource in singular, all lowercase.

- If a resource would have more than 6 permissions, ask whether two of them are the same thing.

- Every new permission gets a label in both languages in `translations/permissions.*.yaml` before the enum case is merged; `PermissionLabelsTest` fails otherwise.
