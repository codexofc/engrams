---
name: playbook-carrier-payout-missing
description: Paiement transporteur en retard : facturé ou pas, conditions de paiement, état Payla du virement, blocage IBAN 48 h, KYC, finance seule agit
type: reference
status: active
verified: 2026-06-05
---

# Le transporteur n'a pas reçu son paiement

Catégorie `payment:payout`. Le transporteur dit qu'un chargement livré il y a X jours n'est pas payé. Il faut savoir si le virement est prévu, parti, ou rejeté, et ne rien promettre.

## Vérifications

1. **Le chargement est-il facturé ?** `hfctl load get <load_id>` : `status` doit être `INVOICED`. En `DELIVERED`, la facture n'est pas encore émise, souvent parce que le POD manque ou que le chargeur n'a pas validé les suppléments. Macro `payout-not-yet-invoiced` avec la raison. Voir [[playbook-load-stuck-in-transit-after-delivery]] si c'est encore `IN_TRANSIT`.

2. **Les conditions de paiement.** `hfctl org get <carrier_org_id>` : `payout_terms` (`D+7`, `D+15`, `D+30` après finalisation de la facture, ou `fast` pour le paiement anticipé payant). Comparer avec `finalized_at` de la facture. Si l'échéance n'est pas passée : macro `payout-terms-explain` avec la date. La moitié des tickets s'arrête là.

3. **L'état du virement.** `hfctl payout get --invoice <invoice_id>`. Champs : `payla_status` (`SCHEDULED`, `SUBMITTED`, `SETTLED`, `RETURNED`, `FAILED`), `scheduled_for`, `submitted_at`, `iban_last4`.

- `SCHEDULED` avec `scheduled_for` passé de plus d'un jour : le lot de paiement n'est pas parti. Escalade finance, pas L2.

- `SUBMITTED` depuis moins de 3 jours ouvrés : en cours, normal en SEPA. Macro `payout-in-progress`.

- `SETTLED` : Payla dit que c'est réglé. Donner `submitted_at` et `iban_last4`, le transporteur vérifie son relevé. S'il maintient, il doit demander à sa banque, on ne peut rien voir de plus.

- `RETURNED` : la banque a renvoyé les fonds (compte clos, IBAN invalide). Le transporteur doit corriger son IBAN dans ses réglages, la finance reprogramme au prochain lot. Macro `payout-returned`.

- `FAILED` : erreur Payla. Escalade finance avec l'identifiant.

4. **IBAN changé récemment.** `hfctl org get` montre `iban_changed_at`. Un changement d'IBAN bloque les paiements 48 h (sécurité, HF-3095, après un cas de fraude par changement d'IBAN). Un virement `SCHEDULED` pendant cette fenêtre attend la fin du délai. Macro `payout-iban-hold`. On ne lève pas ce délai, même pour un client qui insiste.

5. **KYC.** `kyc_state` autre que `VERIFIED` : Payla refuse de payer. Voir [[playbook-kyc-verifid-stuck]].

## Ce qu'on ne fait pas

Le support n'a pas d'accès à Payla. Il ne relance pas un virement, ne change pas un IBAN, ne lève pas un blocage. Tout ça est finance. Le support lit l'état et explique.

## Escalade

File finance de Deskline avec `invoice_id`, `payout_id`, l'état vu et la date. Délai de réponse finance : un jour ouvré. Pour un transporteur Enterprise ou un montant au-dessus de 20 000 EUR, prévenir aussi le responsable de compte.
