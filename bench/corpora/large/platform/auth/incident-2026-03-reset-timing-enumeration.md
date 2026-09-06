---
name: incident-2026-03-reset-timing-enumeration
description: Mars 2026: POST /v1/auth/reset/request prenait 60 ms de plus pour une adresse existante (dispatch synchrone), énumération possible, corrigé, HF-4340
type: project
status: active
verified: 2026-04-02
---

# Incident 2026-03-12 : énumération de comptes par le temps de réponse du reset

## Le signalement

Le 2026-03-12, un chercheur en sécurité indépendant a écrit à `security@halden.example` (adresse du programme de signalement, pas de prime, une page de remerciements) : en envoyant 50 requêtes `POST /v1/auth/reset/request` pour une adresse connue et 50 pour une adresse inventée, la médiane de la première était de 265 ms, celle de la seconde de 205 ms, de façon stable. Il pouvait donc dire si une adresse avait un compte chez nous. Il fournissait le script et les mesures. Il avait raison.

## La cause

Le flux de réinitialisation ([[password-reset-flow]]) avait été écrit pour répondre `202` dans les deux cas et pour exécuter un hachage factice dans le cas « inconnu », de sorte que le coût cryptographique soit égal. Ce qui n'était pas égal : dans le cas « connu », le contrôleur appelait `NotificationDispatcher::dispatch()` de façon synchrone, ce qui écrit deux lignes en base et publie un message sur RabbitMQ, environ 60 ms. Le cas « inconnu » ne faisait rien après le hachage factice.

Le test de timing de la CI n'existait pas encore ; il a été écrit après. La revue de l'extraction d'octobre avait dit « réponse en temps constant » et tout le monde avait compris « même code de statut, même hachage ». Le temps constant, c'est une mesure, pas une intention.

## La correction (HF-4340, déployée le 2026-03-13)

1. Le dispatch de la notification est sorti de la requête : le contrôleur écrit uniquement le jeton dans `password_reset_tokens` (une insertion, 3 ms), et un message Messenger `SendResetEmail` est publié dans les deux cas, connu ou inconnu, avec un drapeau. Le consommateur ne fait rien pour le cas inconnu. Le coût dans la requête est ainsi le même : un hachage, une insertion (factice dans une table tampon pour le cas inconnu, purgée toutes les heures), une publication.

2. Un délai aléatoire n'a pas été ajouté. Le bruit ne masque une différence de médiane que si on ne mesure pas assez ; 60 ms d'écart se retrouvent sous 100 ms de bruit uniforme en quelques centaines de requêtes. Rendre les deux chemins égaux est la seule correction qui tient.

3. Test de CI : 200 requêtes de chaque type sur le runner, échec si les médianes diffèrent de plus de 15 %. Mesuré après correction : 208 ms contre 211 ms.

4. La même analyse a été faite sur les deux autres endpoints qui répondent pareil qu'il y ait un compte ou non : `POST /v1/auth/login` (même hachage factice, pas de dispatch, écart mesuré 2 ms, dans le bruit) et `POST /v1/auth/otp/request` ([[driver-otp-login-server-side]]) qui, lui, avait le même défaut de 45 ms sur les numéros connus et a été corrigé de la même façon dans le même ticket.

## Impact

Inconnu, probablement nul. `auth_events` garde chaque requête de reset avec son IP ; sur 90 jours, aucune IP n'a fait plus de 40 requêtes de reset pour des adresses distinctes, et les 60 requêtes quotidiennes pour des adresses inconnues sont réparties sur des centaines d'IP, ce qui ressemble à des fautes de frappe et à des scans superficiels, pas à une énumération systématique. L'énumération aurait de toute façon révélé « cette adresse a un compte », pas un accès.

Le chercheur a reçu la réponse sous 24 h, la correction sous 48 h, et son nom sur la page de remerciements avec son accord.

## Ce qu'on retient

- « Constant » se vérifie avec un chronomètre, dans la CI, et le test est gardé même s'il est bruyant.

- Tout ce qui se passe dans une requête « qui ne doit rien révéler » doit être identique dans les deux branches, y compris ce qui n'est pas cryptographique : une écriture, une publication, un appel réseau. Le hachage factice était la partie visible du problème et il avait été traité ; le reste ne l'avait pas été.

- L'adresse `security@` avec une vraie personne derrière et une réponse rapide vaut plus qu'un programme de primes qu'on n'a pas les moyens de gérer. Cinq signalements depuis son ouverture en octobre 2025, deux valides, celui-ci était le plus sérieux.
