---
name: vendor-invoices-reconciliation
description: Chaque facture variable (Bipline, Courrix, Verifid, Skyvale, cartographie) est rapprochée de notre compte d'usage le premier jour ouvré, écart toléré 1 %
type: reference
status: active
verified: 2026-07-08
---

## Le principe

Une facture à l'usage se vérifie contre notre usage, pas contre la facture du mois d'avant. Pour chaque fournisseur variable de [[infra-cost-overview-2026]], on a une requête qui produit la quantité facturable du mois depuis nos propres données, et la personne de permanence ([[costs-reviewer-preferences]]) compare le premier jour ouvré. Écart toléré : 1 % ou 50 EUR, le plus grand des deux. Au-delà, un e-mail au fournisseur avec les deux chiffres, avant de payer.

## Les rapprochements

### Par fournisseur

| Fournisseur | Ce qu'il facture | Notre compte | Écart typique |
|---|---|---|---|
| Bipline | segments SMS par pays de destination, numéros longs loués | `notification_deliveries` : `sum(segments)` par `sms_country` sur les livraisons `sent` ou `delivered` du mois | 0,2 à 0,5 % (les `expired` que le contrat dit gratuits) |
| Courrix | e-mails acceptés, IP dédiée au forfait | `notification_deliveries` canal email, statut au moins `sent` | 0,1 % |
| Verifid | vérifications par type (identité, entreprise, document) | `kyc_checks` par `check_type` avec un `provider_ref` | 0 à 0,3 % |

### Par fournisseur, suite

| Fournisseur | Ce qu'il facture | Notre compte | Écart typique |
|---|---|---|---|
| Skyvale | egress par hôte, heures de VM par groupe, requêtes DNS par million, plan engagé | rapport d'usage Skyvale par hôte (le leur) croisé avec nos journaux CDN échantillonnés ; heures de VM depuis nos propres événements de création et suppression | 1 à 2 % sur l'egress (l'échantillonnage), 0 sur les VM |
| fournisseur cartographique | appels d'API par type (géocodage, itinéraire, matrice), tuiles au forfait | compteur `maps.api_calls{type}` de notre proxy | 0,5 % |
| prestataire datacentre | forfait racks, électricité au relevé, mains distantes à l'heure | relevé du compteur électrique visible sur le portail, tickets de mains distantes | 0 sur le forfait ; l'électricité varie avec la saison |

Les fournisseurs fixes (licences, support, amortissement) ne se rapprochent pas, ils se relisent une fois par an à la reconduction.

## Les six écarts trouvés depuis 2025

1. **Bipline, février 2026** : 9 400 messages `expired` facturés, que le contrat déclare gratuits. 610 EUR, crédités le mois suivant après un e-mail. La requête de rapprochement exclut désormais explicitement `expired` et le chiffre est dans le mail.

2. **Skyvale, octobre 2025** : ce n'était pas un écart de facture (la facture était juste) mais le rapport par hôte a révélé les tuiles ([[egress-finding-map-tiles-2025-11]]). Le rapprochement a fait son travail en montrant *où* était l'usage.

3. **Verifid, décembre 2025** : 220 vérifications d'entreprise facturées pour 180 dans `kyc_checks`. Les 40 de plus étaient des relances automatiques de Verifid sur des dossiers incomplets, facturées comme des vérifications. Contractuellement discutable, ils ont crédité la moitié et le paramètre de relance automatique a été coupé (on relance nous-mêmes, gratuitement, depuis le pipeline de notifications).

4. **Fournisseur cartographique, mars 2026** : 1,4 M d'appels de matrice facturés pour 0,9 M comptés. Notre proxy ne comptait pas les appels faits par le job de nuit du calcul d'ETA, qui contournait le proxy avec sa propre clé. La facture était juste, notre compte était faux. Le job passe par le proxy, et une clé d'API du fournisseur hors proxy est maintenant une anomalie recherchée (ils fournissent l'usage par clé).

5. **Prestataire datacentre, août 2025** : 6 heures de mains distantes facturées pour 2 heures de ticket. Erreur de saisie chez eux, corrigée.

6. **Courrix, avril 2026** : l'IP dédiée facturée au prorata depuis le 1er alors qu'elle a été activée le 8. 40 EUR, sous le seuil, signalé quand même parce que c'était le premier mois et qu'on voulait que la règle du prorata soit posée.

Total récupéré : environ 1 300 EUR. Total de temps passé : une heure par mois. Le vrai rendement est le point 2, qui n'était pas un écart.

## Les clauses qu'on relit à chaque reconduction

- **Bipline** : la gratuité des `expired` et des `rejected`, le tarif par pays (a bougé de +6 % en janvier 2026 avec deux mois de préavis contractuel, respecté), la limite de 50 requêtes par seconde, le volume mensuel engagé (300 000, on est à 296 000 en mai, un dépassement se facture à l'unité au même tarif, donc l'engagement ne coûte rien à dépasser légèrement).

- **Courrix** : le prix par tranche de volume (on est dans la tranche 1 à 2 M par mois, la suivante à 2 M baisse le prix unitaire de 12 %, ce qu'on atteindra fin 2026 au rythme actuel), la rotation DKIM incluse, le forfait IP dédiée.

- **Verifid** : le prix par type, ce qui compte comme une vérification (le point 3), la conservation des documents chez eux (30 jours puis suppression, ce que notre classe de rétention exige).

- **Skyvale** : le plan engagé ([[reserved-capacity-decision-2026-01]]), le tarif de l'excédent, la facturation de l'egress vers nos propres racks (gratuite, il a fallu le demander).

- **Fournisseur cartographique** : renouvellement en 2027, le forfait tuiles, la clause d'usage hors ligne pour l'app conducteur.

## Ce qu'on ne rapproche pas et pourquoi

Les frais Payla : c'est un pourcentage du volume réglé, la facturation le rapproche à la transaction dans sa propre réconciliation quotidienne, et le résultat va à la finance. Le refaire ici serait une deuxième version d'un contrôle qui existe.

## La requête type, pour Bipline

Celle qu'on colle dans le mail quand il y a un écart, à adapter aux autres fournisseurs :

```
SELECT sms_country, sum(segments) AS segments, count() AS messages
FROM analytics.notification_daily
WHERE day BETWEEN '2026-05-01' AND '2026-05-31'
  AND channel = 'sms' AND status IN ('sent', 'delivered')
GROUP BY sms_country ORDER BY segments DESC
```

Elle tourne sur l'agrégat quotidien de l'entrepôt, pas sur PostgreSQL, parce que les livraisons de plus de 180 jours ont été purgées côté application et qu'un litige de facture peut remonter à trois mois. Le résultat est collé dans `finops/reconciliations/2026-05-bipline.md` avec le total de la facture, l'écart en segments et en euros, et la décision (payer, contester, noter). Douze fichiers par an et par fournisseur, ce qui est la trace que la finance demandait.
