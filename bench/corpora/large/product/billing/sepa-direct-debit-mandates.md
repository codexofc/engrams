---
name: sepa-direct-debit-mandates
description: SEPA B2B mandates signed in-app with Payla as creditor, J-2 pre-notification, 36-month dormancy, failure codes and actions
type: reference
status: active
verified: 2026-05-19
---

Les prélèvements SEPA représentent 88 % des encaissements chargeurs. On utilise le schéma **SEPA B2B**, pas le schéma Core : pas de droit à remboursement pendant 8 semaines pour le débiteur, ce qui est la seule option compatible avec le fait qu'on reverse aux transporteurs avant d'être payés (voir [[self-billing-carriers]]). La contrepartie : le chargeur doit enregistrer le mandat auprès de sa propre banque, et environ 12 % des premiers prélèvements échouent en `mandate_not_registered`.

## Cycle de vie d'un mandat

1. Signature dans l'app chargeur (`/settings/payment`), formulaire avec IBAN, BIC déduit, raison sociale. Signature électronique par code SMS, horodatage stocké dans `sepa_mandates.signed_at`.
2. Création chez Payla (`POST /v2/mandates`), qui renvoie `mandate_reference` au format `HFRMDT` + 12 chiffres. L'identifiant créancier (ICS) est celui de l'entité facturante, un par pays.
3. Le mandat est `pending_first_debit` jusqu'au premier prélèvement réussi, puis `active`.
4. Un prélèvement rejeté avec `mandate_revoked` ou `mandate_not_registered` passe le mandat en `suspended` et le chargeur reçoit l'email `mandate.action_required` avec le PDF du mandat à transmettre à sa banque.
5. Un mandat sans prélèvement pendant 36 mois devient caduc (`expired`), règle du rulebook SEPA. Le job `ExpireDormantMandates` tourne le 1er de chaque mois.

## Pré-notification

Le rulebook demande une pré-notification 14 jours avant, mais le contrat peut réduire ce délai. Nos CGV le fixent à 2 jours calendaires. L'email `debit.prenotification` part à J-2 à 09:00 avec le montant, la date et la référence de mandat. Un chargeur qui n'a pas reçu la pré-notification peut contester le prélèvement auprès de sa banque même en B2B, donc l'envoi est journalisé dans `notification_log` et le prélèvement n'est soumis que si le journal contient l'envoi.

## Montants et regroupement

Un prélèvement par chargeur et par jour, regroupant toutes les factures échues ce jour-là. Plafond par prélèvement : 50 000 EUR, configurable par compte (`shipper_accounts.max_debit_cents`) parce que certaines banques rejettent au-dessus d'un plafond fixé par le débiteur. Au-delà, le reste est prélevé le lendemain.

## Échecs fréquents

| Code Payla | Cause | Action |
|---|---|---|
| `insufficient_funds` | provision | relance niveau 1 dès le lendemain, voir [[dunning-schedule]] |
| `mandate_not_registered` | banque du chargeur pas informée | email `mandate.action_required`, prélèvement retenté à J+7 |
| `account_closed` | IBAN fermé | mandat suspendu, demande de nouveau mandat |
| `refused_by_debtor` | opposition | mandat révoqué, passage en virement, compte signalé à la finance |

Un IBAN est validé à la saisie par clé de contrôle et par la liste des pays SEPA. Les IBAN non-SEPA (Suisse hors zone SEPA pour les prélèvements B2B, par exemple) ne peuvent pas signer de mandat, ces chargeurs paient par virement.
