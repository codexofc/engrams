---
name: case-ferreira-kyc-name-mismatch
description: February 2026, Ferreira Transportes rejected three times by Verifid for name_mismatch; fix was pre-filling the register name, not a workaround
type: project
status: active
verified: 2026-04-20
---

# Case: Ferreira Transportes, rejected three times for a name

Fictional Portuguese carrier, 12 trucks, Starter plan, `account:kyc`. Tickets 2026-02-03, 02-06 and 02-11, same org.

## What happened

The carrier registered as "Ferreira Transportes" and started verification. Verifid compared the name with the commercial register and found "Ferreira & Filhos, Transportes Rodoviários, Lda." Rejection `name_mismatch`. The owner corrected to "Ferreira e Filhos Transportes", rejected again (the register has "&", the legal form "Lda." was missing, and Verifid's tolerance is not that wide). Third attempt "Ferreira & Filhos Transportes Lda", rejected: "Rodoviários" missing. Each attempt cost him a day of Verifid queue and he could not bid for ten days in total.

His third ticket said, roughly, that he had been running the company for 22 years and knew its name.

## What we did

- Support sent the exact register name, which L2 can see in the Verifid case (`register_name` field). Fourth attempt verified in four hours.

- Support asked the KYC owners why we did not show the register name ourselves. Answer: we could, Verifid returns it on the first rejection, and we were storing it without displaying it.

- HF-3082: after a `name_mismatch`, the restart screen pre-fills the legal name from `register_name` and shows "Use the name exactly as registered". Shipped 2026-03-04.

- HF-3083: the registration form now looks up the company by registration number (Verifid has a lookup endpoint) and pre-fills the legal name and address for the seven countries where the register is available. Shipped in April.

## What we learned

- The customer had the wrong information and we had the right one on file. Support workarounds (telling the name by ticket) hide product gaps; the third ticket is when we noticed.

- Starter customers do not get an account manager, so nobody was watching the ten days without bids. The triage now flags an org with three tickets in ten days whatever the plan.

- "Rejected" with a reason code is not enough of an explanation. The reason must say what to type.

## Figures

`name_mismatch` was 41 % of KYC rejections in Q4 2025. After HF-3082 and HF-3083, 12 % in Q2 2026, and those are mostly companies not in the seven countries covered by the lookup. Median time to `VERIFIED` for new carriers went from 3.1 days to 1.4 days.

## Related

The playbook for KYC states is in the playbooks project; this case is what made its `name_mismatch` step say "give them the register name". Counted in [[case-lessons-recurring-themes-2026-h1]].
