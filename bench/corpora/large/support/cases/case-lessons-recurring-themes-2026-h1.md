---
name: case-lessons-recurring-themes-2026-h1
description: Synthèse des cas support de septembre 2025 à juin 2026 : huit leçons récurrentes, le cas d'origine de chacune et ce qui a changé
type: reference
status: active
verified: 2026-07-07
---

# Ce que les cas nous ont appris, septembre 2025 à juin 2026

Vingt cas écrits sur la période, relus ensemble début juillet 2026 par le support, un backend et le produit. Huit thèmes sont ressortis. Pour chacun : le cas d'origine, les autres cas qui s'y rattachent, ce qu'on a changé, ce qui reste.

## 1. Le périmètre d'une demande se confirme par écrit

Origine : [[case-meridian-agro-bulk-cancel-request]], 17 chargements annulés à tort sur une phrase. Rattaché : [[case-rutten-wrong-pickup-cancel-in-transit]], une référence sans organisation.

Changé : le support n'agit plus sur une description, seulement sur des identifiants ; pas d'action en masse ; `hfctl load list --reference` exige `--org` ; la règle des deux personnes pour l'irréversible. Reste : rien de prévu, la règle tient.

## 2. Lire l'acteur avant de défendre le calcul

Origine : [[case-delacroix-rating-drop-shipper-cancels]], six retraits faits par le chargeur comptés contre le transporteur. Rattaché : [[case-petrov-timezone-pickup-window]], onze retards qui étaient un fuseau.

Changé : `shipper_withdraw` distinct de `withdraw_carrier`, le fuseau de saisie est celui du lieu, la décomposition de la note visible par le transporteur, et une consigne dans le playbook : si tous les événements pénalisants viennent d'un même chargeur ou ont la même durée, chercher une cause commune avant de répondre « le calcul est le même pour tous ». Reste : plafonner la part d'un seul chargeur dans la note (en discussion).

## 3. Une erreur stockée et jamais montrée est une erreur perdue

Origine : [[case-boulanger-cargolink-price-floor]], `last_error` rempli depuis des mois, lu par personne. Rattaché : [[case-aldemar-payout-iban-change-hold]], un blocage juste sans explication à l'écran ; [[case-ferreira-kyc-name-mismatch]], le nom du registre en base et pas à l'écran.

Changé : les erreurs partenaire sur la fiche chargement, la raison du blocage de paiement sur l'écran, le nom légal pré-rempli après un rejet. Reste : un inventaire des champs `*_error` et `*_reason` qui ne sont affichés nulle part, demandé au backend (HF-3230).

## 4. La donnée impossible est un signal, pas du bruit

Origine : [[case-sandoval-shared-phone-two-drivers]], deux traces GPS qui s'éloignent à 90 km/h. Rattaché : [[case-oyelaran-erasure-active-load]], un chauffeur parti encore assigné à un camion qui roule ; [[case-nordvik-gps-battery-complaint]], onze tickets fermés correctement qui cachaient un problème de batterie.

Changé : la requête hebdomadaire `driver_impossible_trips`, la vue des sessions inactives pour les transporteurs, et la règle du tri : trois tickets d'un même transporteur sur un même sous-tag dans le trimestre remontent, même résolus.

## 5. Un 200 n'est pas une preuve, un rejeu n'est pas un diagnostic

Origine : [[case-brenner-webhook-duplicate-loop]]. Rattaché : [[case-havelka-polling-rate-limit]], « votre API est en panne » qui était 1 600 requêtes par minute.

Changé : pas de rejeu sans avoir lu `last_error`, le temps de réponse et la taille de réponse du client dans les livraisons, `Retry-After` en haut de la doc, contact technique direct exigé pour un Business ou Enterprise avec prestataire. Reste : un test de bout en bout que l'intégrateur peut lancer lui-même depuis son écran (HF-3235).

## 6. Le cas normal est celui que l'interface doit gérer

Origine : [[case-marchetti-pod-wrong-load-yard]], deux livraisons dans une cour, 9 % de réattributions évitées une fois la question posée. Rattaché : [[case-bosque-fretzone-duplicate-loads]], un chargeur qui publie à deux endroits et deux dispatcheurs qui acceptent chacun ; [[case-kowalczyk-fleet-locked-out-2025-11]], des téléphones partagés.

Changé : la question « pour quel chargement ? » avant l'envoi d'un document, la déduplication partenaire par référence, l'annonce des règles qui changent la manière de travailler avant la version. Reste : la correspondance floue pour les annonces partenaires sans référence, et l'avertissement « un collègue vient d'accepter une offre similaire » (HF-3215).

## 7. « Pas un bug » mérite quand même un livrable

Origine : [[case-tessalia-invoice-pln-rounding]], une déclaration d'arrondi signée à la place d'une discussion. Rattaché : [[case-feedback-carriers-want-phone-support]], pas de ligne téléphonique mais un rappel sous deux heures ; [[case-ravello-search-radius-confusion]], quatre plaintes, un seul choix d'interface.

Changé : la liste des irritants distingue « accepté, pas de correctif » avec la réponse à donner. L1 sait quoi envoyer sans escalader.

## 8. Un mécanisme de secours qui repose sur une personne n'est pas un secours

Origine : [[case-nordwerk-sso-idp-migration-lockout]], l'admin de dernier recours en congé. Rattaché : [[case-vauclair-double-invoice-2025-11]], une correction manuelle sans précondition qui laisse une facture derrière elle.

Changé : deux admins de dernier recours obligatoires, le repli e-mail activable par le client pour sept jours, `hfctl load undeliver` avec ses vérifications à la place d'un `UPDATE`. Reste : rien d'ouvert.

## Ce qu'on ne retire pas de la liste

La règle « le support ne clique pas pour le client » a été discutée deux fois et confirmée deux fois. Les cas 1 et 6 montrent que l'écran du client contient l'information (le récapitulatif par site, la carte du rayon) que le support n'a pas.

## Méthode

Les cas sont écrits selon [[case-anonymisation-rules]]. Cette synthèse sera refaite en janvier 2027 sur le second semestre ; les thèmes qui ne reviennent pas sortent de la liste, ceux qui reviennent gardent leur numéro.

## Ce que la liste ne contient pas, et pourquoi

Deux thèmes ont été proposés et écartés à la relecture. « Les clients ne lisent pas la documentation » : vrai, inutile, et chaque fois qu'on a déplacé l'information dans l'écran (le `Retry-After`, la raison du blocage de paiement) le problème a disparu, donc c'est une variante du thème 3. « Les petits transporteurs ont besoin de plus d'accompagnement » : vrai aussi, mais c'est un constat commercial, pas une leçon de support ; il est chez le produit avec le rappel sous deux heures comme réponse.

Trois cas de 2026 ne rentrent dans aucun thème et sont restés seuls : un chargeur qui a fait dépendre son processus interne d'un champ de webhook non documenté, un transporteur dont le comptable a contesté le libellé d'un avoir (réglé par la finance en une réponse), et un chauffeur dont le téléphone affichait l'heure d'un autre fuseau à cause d'une carte SIM étrangère (ses positions étaient horodatées deux heures plus tôt, corrigé en 4.9 par l'usage systématique de l'heure serveur pour `received_at`). S'ils reviennent, ils feront un thème 9.
