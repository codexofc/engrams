---
name: webhook-incident-2026-03-relay-listen-stall
description: Incident du 2026-03-04 : relais webhook muet 52 minutes, connexion LISTEN morte sans erreur et balayage de secours filtrant le mauvais statut
type: project
status: active
verified: 2026-03-18
---

# Incident 2026-03-04 : le relais n'écoute plus (HF-3110)

## Chronologie (UTC)

- 09:41 Bascule planifiée de la réplique PostgreSQL vers une nouvelle instance ; PgBouncer est reconfiguré, la connexion directe `DATABASE_URL_LISTEN` du relais pointe vers le primaire, qui n'a pas bougé, mais le pare-feu réseau a été rechargé pour l'occasion.

- 09:43 Dernière livraison webhook réussie enregistrée.

- 10:05 Un intégrateur Enterprise (un chargeur, canal partagé) signale ne plus recevoir de `load.dispatched`. L1 vérifie un abonnement : les livraisons sont en `PENDING`, aucune tentative.

- 10:12 L2 regarde le pod du relais : vivant, aucune erreur dans les logs, la dernière ligne « received NOTIFY » date de 09:43. Le CronJob `--sweep` a bien tourné toutes les minutes et sortait « 0 rows ».

- 10:20 Backend pagé (critère : plus de 10 organisations touchées, en fait toutes). Redémarrage du pod du relais. Les livraisons repartent immédiatement, 6 100 lignes `PENDING` écoulées en 4 minutes.

- 10:35 Incident clos côté client. 52 minutes sans livraison, aucune perte : tout était dans l'outbox.

## Causes

Deux, l'une cachant l'autre.

**La connexion LISTEN était morte sans que le client le sache.** Le rechargement du pare-feu a coupé les connexions TCP inactives établies avant. La connexion `LISTEN` du relais est inactive par nature (elle attend). Sans keepalive TCP, le client ne voit rien : `pg_notifies()` renvoie « rien de nouveau » pour toujours. Le serveur, lui, avait oublié la connexion. Le relais était en bonne santé selon toutes ses sondes, qui ne testaient que le processus.

**Le balayage de secours ne rattrapait pas.** `--sweep` devait précisément couvrir « notification perdue ». Il sélectionnait `status = 'FAILED' AND next_attempt_at <= now()`, c'est-à-dire les relances, et non `status = 'PENDING'`. Les lignes neuves, jamais tentées, n'étaient prises que par le chemin `NOTIFY`. Une régression de HF-3060 (janvier), quand le balayage a été réécrit pour partager le code des relances : le `IN ('PENDING','FAILED')` est devenu `= 'FAILED'` dans le refactor, et aucun test ne couvrait « le NOTIFY ne vient pas ».

## Correctifs

- Keepalive TCP sur `DATABASE_URL_LISTEN` (`keepalives=1 keepalives_idle=30 keepalives_interval=10 keepalives_count=3` dans le DSN) et, côté application, un `SELECT 1` sur la connexion LISTEN toutes les 60 s ; si ça échoue, le relais sort avec un code d'erreur et Kubernetes le redémarre.

- Le balayage reprend `status IN ('PENDING','FAILED')`, avec un test d'intégration qui insère une ligne sans passer par le trigger `NOTIFY` et vérifie qu'elle est livrée par le balayage seul.

- Une sonde de disponibilité du relais fondée sur les données : `max(now() - created_at)` des lignes `PENDING` doit rester sous 2 minutes, sinon le pod est déclaré non prêt et une alerte `WebhookRelayStalled` part au bout de 5 minutes ([[webhook-delivery-metrics-and-slo]]). C'est cette alerte qui aurait dû exister depuis le début.

## Ce qu'on a appris

- Une sonde qui teste « le processus tourne » ne teste rien d'utile pour un consommateur d'événements. La bonne sonde, c'est « le travail avance ».

- Un chemin de secours qu'on n'exerce jamais ne marche pas. Le balayage tournait toutes les minutes depuis janvier et personne n'avait vu qu'il ne servait à rien, parce que le `NOTIFY` marchait.

- Le support a trouvé le problème en 20 minutes grâce à l'écran des livraisons (`PENDING`, zéro tentative). Sans cet écran, on aurait cherché chez le client.

## Suite

Le relais a été rendu redondant en juin 2026 (deux pods, `FOR UPDATE SKIP LOCKED` fait le partage), ce qui n'aurait pas aidé ici (les deux auraient eu la même connexion morte) mais couvre les pannes de pod. Le vrai filet, c'est l'alerte sur l'âge des `PENDING`.

## Détails techniques du correctif LISTEN

Le relais est un processus PHP long (`app:outbox:relay`) avec une connexion PDO dédiée pour `LISTEN outbox_new`, hors PgBouncer parce que PgBouncer en mode transaction ne relaie pas les notifications. La boucle principale appelait `pgsqlGetNotify(PDO::FETCH_ASSOC, 1000)` et, sans notification en une seconde, repartait. Une connexion morte côté serveur ne fait pas échouer `pgsqlGetNotify` : elle renvoie `false` comme un délai écoulé. La boucle ne pouvait pas faire la différence.

Le correctif ajoute, toutes les 60 boucles (donc environ toutes les minutes), un `SELECT 1` sur cette même connexion. Une connexion coupée lève alors une `PDOException` (`server closed the connection unexpectedly`), attrapée par la boucle, qui journalise, ferme proprement et sort avec le code 3. Le Deployment a `restartPolicy: Always`, le pod redémarre en quelques secondes, se reconnecte, refait `LISTEN`, puis exécute un balayage complet avant de reprendre l'écoute, pour rattraper ce qui est arrivé pendant la coupure. Le keepalive TCP dans le DSN est le second filet : il détecte la connexion morte en une minute même si l'application ne fait rien.

## Ce qu'on a vérifié après

- Test d'intégration : le test démarre le relais contre une base locale, insère une ligne en contournant le trigger, et vérifie qu'elle est livrée par le balayage en moins de 90 s. Il tourne dans la CI du dépôt API depuis le 6 mars.

- Test de chaos : en staging, un `iptables` coupe les connexions établies vers PostgreSQL pendant que le relais tourne ; le relais doit redémarrer et livrer une ligne insérée après la coupure en moins de 3 minutes. Fait à la main le 10 mars, puis ajouté à l'exercice mensuel de l'équipe plateforme.

- Le CronJob `--sweep` a été gardé même si le relais fait maintenant son propre balayage au démarrage, parce qu'il couvre le cas où le relais ne tourne pas du tout (déploiement raté, nœud perdu). Il a été rendu visible : chaque exécution écrit le nombre de lignes reprises dans une métrique, `hf_webhook_sweep_rows_total`, qui doit rester proche de zéro en régime normal ; un chiffre élevé la nuit veut dire que le relais dort et que personne ne s'en est aperçu.
