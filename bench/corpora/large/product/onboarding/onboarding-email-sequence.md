---
name: onboarding-email-sequence
description: Seven-message carrier onboarding sequence, conditional on the step reached, open and click rates, day 2 content moved in-app
type: project
status: active
verified: 2026-05-20
---

## Séquence

Envoyée par `OnboardingSequencer`, un message par jour au plus, à 09:00 dans le fuseau du transporteur, dans la langue du compte. Chaque message a une condition ; s'il n'y a plus rien à dire, rien ne part.

| Jour | Message | Condition | Ouverture | Clic |
|---|---|---|---|---|
| 0 | bienvenue, ce qu'il faut préparer (licence, assurance, IBAN) | tous | 78 % | 31 % |
| 1 | « votre licence pour enchérir », avec la mention de la copie conforme | étape 3 non passée | 54 % | 22 % |
| 2 | « société vérifiée, il reste la licence » ou « vous pouvez enchérir » selon l'étape | étape 2 passée | 41 % | 15 % |
| 4 | chargements ouverts sur vos lignes (les 5 plus proches) | étape 3 passée, aucune enchère | 63 % | 38 % |
| 7 | « comment se passe le paiement » (autofacturation, IBAN) | étape 5 non passée | 47 % | 19 % |
| 14 | rappel licence ou assurance | étape 3 ou 4 non passée | 33 % | 9 % |
| 21 | dernier message, appel proposé | aucune enchère | 29 % | 7 % (demande d'appel) |

Taux mesurés sur la cohorte de mars 2026 (5 480 comptes, voir [[activation-funnel-q1-2026]]).

## Ce qu'on a appris

- Le message du jour 4 (chargements sur vos lignes) est celui qui fait enchérir : 38 % de clics et 60 % des cliqueurs enchérissent dans la journée. C'est ce qui a inspiré [[first-load-nudge-experiment]].
- Le message du jour 2 était le plus important sur le papier (il explique que la société vérifiée ne suffit pas) et le moins lu. La conclusion des appels de [[support-findings-onboarding-2026-04]] a été de mettre cette information dans l'écran de fin d'étape 2 plutôt que d'améliorer l'email. Le message reste, mais on ne compte plus sur lui.
- Le jour 21 avec proposition d'appel : 7 % de demandes, 44 % de ces appels aboutissent à une première enchère dans la semaine. Un réviseur y passe environ 3 heures par semaine.

## Règles

- Jamais plus d'un email par jour, séquence incluse ; si une notification transactionnelle (document rejeté, par exemple) est partie le matin, le message de séquence du jour est décalé au lendemain.
- Désinscription de la séquence en un clic, sans désinscrire des notifications transactionnelles. 2,1 % se désinscrivent, presque tous après le jour 7.
- Les textes sont dans `notifications/templates/onboarding/*.mjml`, revus par une personne de langue maternelle pour PL et RO (les premières versions roumaines venaient d'une traduction automatique et deux transporteurs l'ont fait remarquer).

## Ce qu'on ne fait pas

Pas de SMS dans la séquence. Testé en 2025 sur 800 comptes polonais : +3 points d'envoi de licence, mais 11 plaintes pour spam et un coût de 0,08 EUR par SMS. Le SMS est réservé aux codes de signature et au rappel d'expiration à 3 jours.
