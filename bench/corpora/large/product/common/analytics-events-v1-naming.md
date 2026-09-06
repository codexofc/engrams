---
name: analytics-events-v1-naming
description: The 2024 screen_element_action event naming, free-form properties and no registry, and why it was replaced in February 2026
type: reference
status: archived
superseded_by: [[analytics-events-naming]]
verified: 2025-12-05
---

Until February 2026 product events were named `<screen>_<element>_<action>` in snake case: `bid_form_submit_clicked`, `onboarding_page_button_clicked`, `invoice_list_row_opened`. Properties were free-form, no registry, no envelope beyond a user id and a timestamp.

Problems that accumulated:

- 640 distinct event names by the end of 2025, of which 210 had fewer than 100 occurrences in the year, and 90 were duplicates with different spellings (`bid_form_submit_clicked` and `bidform_submit_click`).
- Screen-based names broke every time a screen was redesigned; the bid form rewrite of September 2025 silently stopped 14 events.
- No experiment context on events, so every experiment analysis had to join on flag assignment tables by timestamp, which was wrong whenever a unit changed variant.
- Three events carried email addresses in properties, found during a data protection review.

The replacement convention is [[analytics-events-naming]]. Old events were mapped to new ones in the warehouse for the 2025 history where a mapping existed (about 60 % of volume); the rest is kept in the raw events table under the old names and not modelled.
