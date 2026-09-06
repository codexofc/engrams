---
name: incident-process-severity-levels
description: Security incidents are SEV1 to SEV3, declared in #inc-<ticket> with a commander and a scribe; SEV1 pages the rota and head of engineering, PM in 5 days
type: reference
status: active
verified: 2026-04-30
---

# Processus d'incident sécurité

Version en vigueur depuis février 2026 (HF-2109). La version antérieure est archivée dans [[incident-process-v1]]. Le processus vaut pour les incidents de sécurité et de données ; les incidents de disponibilité pure suivent le processus de l'astreinte plateforme, qui est proche mais pas identique (pas de volet notification).

## Déclarer

N'importe qui déclare. Pas besoin d'être sûr, pas besoin d'être de garde. La commande `/incident sécurité <titre>` dans le chat crée :

- un ticket YouTrack `HF-xxxx` avec l'étiquette `security-incident` et un champ `Severity` ;

- un canal `#inc-HF-xxxx` avec le déclarant, la rota sécurité du moment et, pour un SEV1, le responsable technique et le DPO ;

- un message épinglé avec les rôles à remplir et le lien vers cette note.

Une fausse alerte se clôt avec le motif « faux positif » et coûte dix minutes. Un vrai incident déclaré deux heures trop tard coûte beaucoup plus. On ne reproche jamais une déclaration.

## Sévérités

| | Définition | Exemples | Qui est prévenu | Post-mortem |
|---|---|---|---|---|
| **SEV1** | Données personnelles ou secrets de production exposés hors de l'entreprise, ou perte de contrôle d'un composant de production | bucket public, compte staff compromis, clé de signature fuitée | rota sécurité (page), responsable technique, DPO, direction | obligatoire, 5 jours ouvrés, revue en réunion |
| **SEV2** | Exposition interne ou limitée, contrôle affaibli sans perte | secret dans un log de build interne, permission excessive exploitée par un collègue, dépendance malveillante détectée avant exécution | rota sécurité (page en heures ouvrées, sinon le matin) | obligatoire, 5 jours ouvrés, écrit |
| **SEV3** | Pas d'exposition, un contrôle a échoué et a été rattrapé | phishing signalé et non cliqué, scanner de secrets qui a bloqué un commit, tentative de force brute contenue | rota sécurité (ticket) | court, dans le ticket |

La sévérité est fixée par le commandant à la déclaration et **peut monter, jamais descendre** pendant l'incident. Si elle paraît trop haute à la fin, le post-mortem le dit et on apprend à mieux estimer.

Si des données personnelles peuvent être concernées, la procédure de notification à 72 h du projet conformité démarre en parallèle, avec son propre compte à rebours.

## Rôles

- **Commandant** : la personne de la rota sécurité, ou la première personne compétente disponible qui prend le rôle en l'écrivant dans le canal. Décide, coordonne, ne répare pas elle-même si quelqu'un d'autre peut. Une seule à la fois ; un passage de relais s'écrit dans le canal.

- **Scribe** : tient la chronologie horodatée dans le canal, avec l'outil décrit dans [[incident-timeline-tooling]]. Rôle séparé du commandant dès qu'on est deux.

- **Communication** : pour un SEV1, une personne qui rédige les messages externes à partir de [[incident-comms-templates]] et les fait valider par le commandant. Pour les SEV2 et SEV3, le commandant s'en charge.

- **Répondants** : tout le monde d'autre, qui fait ce que le commandant demande et écrit ce qu'il fait dans le canal avant de le faire, quand c'est possible.

## Pendant l'incident

1. **Contenir** avant de comprendre : révoquer, isoler, couper. On peut toujours rouvrir. La question à se poser est « qu'est-ce qui limite l'exposition maintenant », pas « quelle est la cause ».

2. **Préserver** : avant de redémarrer ou de supprimer quoi que ce soit, copier les logs, les métadonnées, l'état. Le commandant dit explicitement « on peut maintenant nettoyer ».

3. **Évaluer l'exposition** : quelles données, combien de personnes, depuis quand, jusqu'à quand. Écrire les estimations avec leur incertitude dans la chronologie.

4. **Communiquer** : un point dans le canal toutes les 30 minutes pour un SEV1, même pour dire « rien de nouveau ». Les clients et l'autorité selon le projet conformité.

5. **Clore** l'incident quand l'exposition est terminée et que les correctifs immédiats sont en place. La cause profonde et les correctifs structurels appartiennent au post-mortem.

## Après

Le post-mortem suit [[postmortem-template-rules]]. Il est **sans blâme** : on nomme des rôles et des systèmes, pas des personnes, et une personne qui a fait une erreur raisonnable dans un système qui la permettait n'est pas le sujet. La revue d'un SEV1 se fait en réunion de 45 minutes ouverte à toute l'ingénierie, sous 10 jours ouvrés.

Les actions du post-mortem sont des tickets avec une échéance, suivis dans la revue mensuelle sécurité. Une action sans ticket n'existe pas.

## Chiffres

Depuis février 2026 : 2 SEV1, 4 SEV2, 11 SEV3, 6 faux positifs. Délai médian entre déclaration et premier geste de containment : 14 minutes. Le retour d'expérience transversal est dans [[incidents-lessons-2025-2026]].

## Le canal de statut pour les SEV1

Un SEV1 attire des gens qui ne répondent pas à l'incident mais ont besoin de savoir : direction, commercial du compte concerné, support. Ils ne vont pas dans `#inc-HF-xxxx`, où leurs questions ralentissent les répondants. Ils vont dans `#inc-HF-xxxx-status`, créé automatiquement pour un SEV1, où la personne au rôle communication poste le point des 30 minutes et répond aux questions. Le commandant ne lit pas ce canal pendant l'incident. Mis en place après le bucket public, où trois questions « quand est-ce que c'est réglé » dans le canal principal ont fait perdre le fil au scribe.
