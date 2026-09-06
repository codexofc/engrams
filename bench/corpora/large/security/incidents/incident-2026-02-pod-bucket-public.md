---
name: incident-2026-02-pod-bucket-public
description: On 2026-02-12 hf-documents was publicly listable for 6 h 40 after a copied module carried public-read; 2 300 drivers' PODs exposed, authority told at 51 h
type: project
status: active
verified: 2026-03-10
---

# Bucket de documents rendu public (février 2026)

Post-mortem `2026-02-12-documents-bucket-public.md`. **SEV1**, premier incident traité avec le nouveau processus ([[incident-process-severity-levels]]) et le premier avec notification à l'autorité.

## Résumé

Une modification de la politique de cycle de vie du bucket `hf-documents` (stockage objet compatible S3 chez notre hébergeur) a été appliquée avec un gabarit qui incluait une ACL `public-read` au niveau du bucket. Pendant 6 h 40, n'importe qui connaissant l'URL du bucket pouvait lister et lire son contenu : preuves de livraison (photos de bordereaux avec signatures et parfois noms manuscrits de réceptionnaires), documents transporteurs (licences, attestations d'assurance). Le bucket contenait 1,4 million d'objets ; les journaux d'accès montrent 3 requêtes de listage et 212 lectures d'objets depuis 2 IP non identifiées, toutes dans le préfixe `pod/2026/02/`, soit environ 2 300 chauffeurs concernés par les objets réellement lus.

## Chronologie (UTC)

- **2026-02-12 09:14** `[ticket]` : HF-2117, réduire les coûts de stockage en passant les PODs de plus de 2 ans en classe froide. La modification est une politique de cycle de vie.

- **09:52** `[cloud]` : la politique est appliquée via l'outil d'infrastructure. Le module utilisé pour le bucket a été copié depuis celui du bucket `hf-public-assets` (logos, images du site), qui porte légitimement `acl = "public-read"`. La ligne n'a pas été retirée. La MR a été relue par une personne, qui a regardé la politique de cycle de vie et pas le reste du diff.

- **09:52 à 16:32** : bucket public.

- **14:10** `[log]` : première requête de listage depuis une IP inconnue. Deux autres à 14:12 et 15:40. 212 `GetObject` entre 14:11 et 16:20.

- **16:25** `[chat]` : un chercheur en sécurité écrit à `security@halden.example` : « votre bucket hf-documents est listable ». Le message est vu à 16:28.

- **16:29** `[chat]` : `/incident sécurité bucket documents public`, SEV1, commandant nommé, scribe nommé, DPO ajouté au canal.

- **16:32** `[cloud]` : ACL remise à `private` à la main dans la console. Fin de l'exposition.

- **16:35** : « on préserve avant de nettoyer » : journaux d'accès du bucket copiés dans `hf-compliance/incidents/HF-2119/`.

- **17:10** : `compliance:breach:open` (projet conformité), `aware_at = 16:25`.

- **17:40** : première estimation : objets lus, préfixe, nombre de chargements, nombre de chauffeurs distincts (par `assignments.driver_id` des chargements concernés) : 2 288.

- **2026-02-13 09:00** : décision DPO : `high_risk` (photos de signatures, noms manuscrits, documents d'identité professionnelle). Notification aux transporteurs concernés (41) à partir de 10:30, tous joints le 13 avant 18:00, soit 30 h après `aware_at`. Texte depuis [[incident-comms-templates]].

- **2026-02-14 19:20** : notification à l'autorité, 51 h après `aware_at`.

- **2026-02-14** : le chercheur est remercié, la question d'une récompense est posée ; on n'a pas de programme, on lui a proposé un paiement au titre de la divulgation, accepté.

## Cause

Le compte de stockage n'avait pas de blocage global des ACL publiques, donc une ACL `public-read` posée sur n'importe quel bucket prenait effet. Aucun contrôle ne comparait l'état effectif des buckets à un état attendu. La copie d'un module d'un bucket public vers un bucket privé était une action ordinaire que rien n'empêchait ni ne détectait.

## Ce qui a bien marché

- Le processus : 4 minutes entre la lecture du message et la déclaration, 7 minutes jusqu'au containment, un scribe dès le début. La chronologie ci-dessus vient du journal du scribe sans reconstruction.

- Les journaux d'accès du bucket étaient activés (depuis 2024, pour la facturation), sans quoi l'estimation d'impact aurait été « tout le bucket ».

- Le chercheur a écrit à une adresse qui existait et était lue.

## Ce qui a mal marché

- 6 h 40 sans détection interne. On a appris l'exposition d'un tiers.

- Le bucket `hf-public-assets` et `hf-documents` partageaient un gabarit de module, différenciés par une seule ligne.

- La notification à l'autorité a pris 51 h : le formulaire demandait des informations que personne n'avait préparées (voir l'exercice décrit dans [[security-incident-drill-2026-03]], organisé à la suite).

## Actions

- **Blocage des accès publics au niveau du compte** de stockage pour tous les buckets sauf `hf-public-assets`, posé le 2026-02-13. Une ACL publique sur un autre bucket est désormais refusée par l'hébergeur.

- **Vérification de dérive** : `storage:policy-check` toutes les 10 minutes compare ACL, politique et blocage public de chaque bucket à `config/storage/buckets.yaml` ; un écart page la rota. Livré le 2026-02-18.

- **Sonde externe** : depuis une machine hors de notre réseau, tentative de listage anonyme de chaque bucket toutes les heures ; un succès page. Livré le 2026-02-20. C'est la détection qu'on n'avait pas.

- **Modules séparés** pour buckets publics et privés dans l'infrastructure, le module privé n'a pas de paramètre `acl`.

- **Objets PODs servis uniquement par URL présignée** de 15 minutes depuis l'API, plus jamais par un chemin de bucket direct, même privé (déjà le cas pour 90 % des accès, les 10 % restants étaient un vieux chemin du back-office).

- **Préparation de la notification** : identifiants du portail de l'autorité dans le vault, requête d'estimation prête, DPO dans le canal dès la déclaration (repris par le projet conformité).

Ticket incident : HF-2119. Ticket d'origine : HF-2117, dont la modification a été refaite proprement le 2026-02-20.

## Chiffres finaux

1,4 million d'objets exposés potentiellement, 212 lus, 2 288 chauffeurs concernés par les objets lus, 41 transporteurs notifiés en 30 h, autorité notifiée en 51 h, 6 h 40 d'exposition, 7 minutes de containment après déclaration, 6 actions toutes closes au 2026-03-10.
