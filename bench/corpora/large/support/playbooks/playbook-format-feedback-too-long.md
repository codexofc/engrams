---
name: playbook-format-feedback-too-long
description: Retour L1 (février 2026) : playbooks trop longs, pourquoi avant quoi, d'où le format en vérifications numérotées avec commande et sortie
type: feedback
status: active
verified: 2026-03-02
---

# Retour L1 : les playbooks sont trop longs

Rétrospective support du 2026-02-09, les six agents, une heure. Sujet unique : pourquoi les playbooks écrits par L2 et le backend n'étaient pas utilisés par L1, alors qu'ils existaient pour 80 % des tickets.

## Ce que L1 a dit

- « Je cherche la commande, elle est au milieu d'un paragraphe. »

- « Le playbook commence par expliquer l'architecture. Le client attend. »

- « Il y a trois manières de faire la même chose et il ne dit pas laquelle prendre. »

- « Il dit « vérifier que le chauffeur est bien assigné » mais pas comment. »

- « Quand je ne sais pas si je peux, je n'ose pas et j'escalade. »

- « Certains sont en anglais avec des captures en français et l'inverse. »

Mesure à l'appui : sur janvier 2026, 34 % des tickets `load:stuck` ont été escaladés en L2 alors que le playbook les couvrait. L2 les a résolus en appliquant... le playbook.

## Ce qu'on a changé

Décidé le jour même, appliqué sur les 24 playbooks en trois semaines :

- **Vérifications numérotées**, dans l'ordre où on les fait. Chaque vérification : la commande exacte, le champ à lire, ce qu'on attend, la sortie (macro, résolu, étape suivante, escalade).

- **Le « pourquoi » en bas**, en trois lignes maximum, sous un titre « Pourquoi » ou dans la dernière section. Personne ne le lit pendant le ticket, on le lit après.

- **Une section « Ce qu'on ne fait pas »** dans chaque playbook. C'est la section qui a le plus baissé les escalades inutiles : savoir qu'on n'a pas le droit évite de se demander si on peut.

- **Une seule manière de faire.** Quand deux commandes existent, le playbook nomme celle du support. L'autre est mentionnée seulement si elle a une raison d'exister.

- **Les macros nommées** dans le playbook, pour ne pas les chercher dans Deskline.

- **Une langue par playbook**, choisie par l'auteur, pas de mélange de captures. Un playbook en français peut citer des identifiants et des messages d'erreur en anglais, c'est la règle de la maison.

- **Deux minutes de lecture**, environ 600 mots. Au-delà, on coupe en deux playbooks liés, ou on garde une version longue à part pour les procédures rares et lourdes (l'annulation à deux, la remise en transit).

## Résultat

Mars 2026 : escalades L2 sur `load:stuck` à 14 %. Avril : 11 %. La [[playbook-driver-cannot-log-in]] et la [[playbook-load-stuck-dispatched-no-pickup]] ont été les deux premières réécrites parce qu'elles portaient la moitié du volume.

## Ce qu'on n'a pas fait

Pas de générateur de playbook depuis un gabarit, pas d'outil : des fichiers markdown dans le dépôt, relus en revue comme du code. Pas de vidéo. Pas de traduction systématique, on n'a pas les bras.
