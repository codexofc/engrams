---
name: security-oncall-rota
description: One engineer per week from a pool of 6, paged by 9 named alerts only, handles advisories, secret-scan hits and incident command, written Monday handover
type: reference
status: active
verified: 2026-04-24
---

# Rota sécurité

Distincte de l'astreinte plateforme (qui gère la disponibilité). Une personne par semaine, du lundi 10:00 au lundi 10:00, tirée d'un groupe de six volontaires venus de l'API, du front, de l'ops et des intégrations. Pas d'astreinte de nuit rémunérée : la rota est jointe la nuit par les pages listées ci-dessous, rarement (4 pages de nuit en 6 mois), et le reste attend le matin.

## Ce que fait la personne de rota

- **Répond aux pages** (liste ci-dessous) en 15 minutes, dans le canal, par un humain.

- **Commande les incidents sécurité** jusqu'à ce que quelqu'un de plus compétent sur le sujet prenne le relais, ce qui s'écrit dans le canal (processus dans le projet incidents).

- **Trie les avis de sécurité** sur les dépendances sous 2 jours ouvrés ([[dependency-updates-policy]]).

- **Traite les hits du scanner de secrets** en CI et en sortie de CI ([[secrets-handling-conventions]]) : vrai secret, on tourne et on déclare ; faux positif, on ajoute l'exception avec la justification.

- **Lit le rapport hebdomadaire** du lundi : dry-run de rétention du projet conformité, secrets de plus de 330 jours, MR de dépendances de plus de 14 jours, refus d'autorisation par utilisateur au-dessus du seuil.

- **Répond à `security@halden.example`** et à `#signalement-securite` dans la journée.

- **Anime la revue mensuelle** si elle tombe dans sa semaine : actions de post-mortem ouvertes, avis en cours, diff SBOM.

Ce n'est pas la personne de rota qui fait les revues de code sécurité ni les modèles de menace : ça, c'est le programme des référents ([[security-champions-program]]).

## Les alertes qui pagent

Neuf, et la liste est fermée : en ajouter une passe par une MR sur `halden-security/alerts.yaml` avec le runbook lié.

| Alerte | Source | Runbook |
|---|---|---|
| `breakglass.used` | audit | projet IAM, procédure break-glass |
| bucket listable depuis l'extérieur | sonde externe | projet incidents, bucket public |
| dérive de politique de stockage | `storage:policy-check` | idem |
| secret détecté dans une sortie de CI | scanner CI | [[secrets-handling-conventions]] |
| connexion staff depuis un pays inconnu | audit | vérifier avec la personne, sinon révoquer |
| `apikey.bruteforce_suspected` au-delà de 5 préfixes en 1 h | audit | projet IAM |
| détecteur de credential stuffing | `CredentialStuffingDetector` | projet IAM |
| échec de la sonde d'isolation des tenants | `tenancy:probe` | projet IAM |
| refus d'autorisation > 50/h pour un même utilisateur | Loki | contacter l'utilisateur ou l'intégrateur |

Tout le reste (avis, rapports, hits pre-commit) est un ticket ou un message, pas une page.

## Passation

Lundi 10:00, 15 minutes en visio, et une **note écrite** dans `halden-security/rota/2026-W17.md` : incidents ouverts, avis en cours, hits non résolus, ce qui a été bizarre sans être un incident. La note écrite est ce qui permet à la personne suivante de ne pas redécouvrir un faux positif déjà analysé. Vingt lignes en général.

## Charge mesurée

Sur les 12 semaines de février à avril 2026 : médiane 3 h 30 par semaine, maximum 14 h (la semaine du bucket public), 9 pages au total dont 4 la nuit ou le week-end (2 vraies, 2 faux positifs de la sonde externe pendant une maintenance réseau de l'hébergeur).

## Règles

- On ne fait pas deux semaines de suite.

- La personne de rota n'est pas en même temps de rota plateforme.

- Une page sans réponse en 15 minutes va au suppléant (la personne de la semaine précédente).

- On peut échanger sa semaine, à condition de l'écrire dans le calendrier partagé avant le lundi.

Les préférences des gens qui tiennent la rota sont dans [[security-team-preferences]].
