---
name: partner-fretzone-contract-quirks
description: Clauses Fretzone qui pèsent sur le code : exclusivité 2 h, marge affichée 3 %, commissions, non-démarchage 12 mois, quota 20 000 appels par jour
type: reference
status: active
verified: 2026-05-15
---

# Fretzone : ce que le contrat impose au code

Le contrat lui-même est chez le juridique. Voici les clauses qu'il faut connaître pour comprendre pourquoi le code fait ce qu'il fait, et pour répondre aux chargeurs et transporteurs qui trouvent ça bizarre.

## Exclusivité de 2 heures

Une annonce publiée par un chargeur Fretzone n'est visible par les partenaires (nous) que 2 heures après sa publication. Leur API ne la renvoie pas avant. Nos transporteurs voient donc les annonces Fretzone avec au moins 2 heures de retard sur les transporteurs Fretzone, et souvent les meilleures sont déjà parties. C'est le prix d'accès à leur volume. L'aide transporteur le dit ; le support a la macro `fretzone-exclusivity`.

Dans l'autre sens, **pas** d'exclusivité : nos annonces sont visibles par leurs transporteurs dès leur poll suivant, en même temps que par les nôtres. Asymétrique, négocié ainsi parce qu'on voulait leur volume et qu'ils voulaient nos annonces.

## Marge affichée de 3 %

Fretzone affiche à ses transporteurs un prix « à partir de » égal à notre prix indicatif moins 3 %. Le transporteur Fretzone mise donc souvent 3 % sous ce que le chargeur attendait, et le chargeur voit chez nous une offre plus basse que son prix et se demande pourquoi. Ce n'est pas un bug ([[partner-load-field-mapping-rules]]) et le chargeur reste libre de refuser. Macro `fretzone-displayed-margin`.

## Commission

Sur chaque chargement attribué à un transporteur Fretzone via notre plateforme, un pourcentage fixe du montant de l'offre acceptée, facturé par Fretzone mensuellement. Et sur chaque annonce Fretzone attribuée à un transporteur Halden, un pourcentage que **nous** facturons à Fretzone. La réconciliation ([[partner-reconciliation-nightly-job]]) calcule les deux ; le relevé mensuel qu'on leur envoie et celui qu'on reçoit doivent coïncider à 2 % près, sinon un ticket chez eux.

## Interdiction de recontacter

Un transporteur Fretzone rencontré via une attribution ne doit pas être démarché par nous pendant 12 mois. Concrètement : les transporteurs fantômes ([[partner-bid-relay-and-shadow-carriers]]) créés pour Fretzone portent `no_solicitation_until` et sont exclus des campagnes d'e-mail et des exports commerciaux. Le chauffeur qui exécute sur notre app reçoit le SMS de connexion et rien d'autre.

Symétriquement, Fretzone ne démarche pas nos chargeurs. On n'a aucun moyen de le vérifier ; un chargeur nous l'a signalé une fois, le commercial a géré.

## Quota d'appels

20 000 appels par jour sur leur API, tous endpoints confondus, au-delà des 429 jusqu'à minuit heure de Paris. Le pull toutes les 5 minutes avec pagination consomme environ 3 500 appels par jour ; les propositions et statuts environ 2 000. Marge confortable, mais un incident de boucle en avril 2026 a consommé le quota en trois heures ([[partner-incident-2026-04-fretzone-price-drift]]). Depuis, un compteur Redis `partners:fretzone:calls:<date>` bloque nos propres appels à 18 000 et alerte.

## Préavis d'API

Fretzone doit annoncer 60 jours avant tout changement incompatible de son API. Ils l'ont fait une fois (passage de `date_enlevement` en chaîne à un objet avec fenêtre, janvier 2026, prévenu en novembre 2025). On doit la même chose pour notre flux. Notre flux est versionné dans l'URL (`/v2/partners/fretzone/feed`) ; une v3 signifierait un préavis.

## Données

On peut afficher le nom et la note Fretzone d'un transporteur Fretzone à nos chargeurs ; on ne peut pas les stocker au-delà de la fin du chargement plus 90 jours, ni les exporter, ni les utiliser dans notre propre calcul de note. Les transporteurs fantômes sont exclus de la dimension transporteur de l'entrepôt et purgés par un job à 90 jours après leur dernier chargement.

## Durée et sortie

Contrat annuel, tacite, préavis de 3 mois. En cas de sortie, on doit supprimer les annonces rapatriées et les transporteurs fantômes sous 30 jours ; `app:partners:fretzone:offboard` existe et est testé en staging deux fois par an, ce qui paraît excessif jusqu'au jour où on en aura besoin.
