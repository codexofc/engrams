---
name: egress-finding-map-tiles-2025-11
description: Novembre 2025: 5 800 EUR/mois d'egress CDN pour les tuiles à cause d'un Cache-Control: no-store oublié depuis 2024, une ligne, −4 200 EUR/mois, HF-4710
type: project
status: active
verified: 2025-12-19
---

# Les tuiles de carte qui coûtaient 5 800 EUR par mois

## Le constat

En préparant la première version de la table de coûts par service ([[per-service-cost-table-q2-2026]] en est la version stabilisée), la ligne « Skyvale egress » ressortait à 7 300 EUR par mois, en hausse de 40 % sur un an alors que le nombre d'utilisateurs du dispatch avait augmenté de 20 %. Le rapport d'usage Skyvale par hôte : `tiles.halden.example` faisait 79 % de l'egress, soit 5 800 EUR et environ 48 TB par mois. Le serveur de tuiles, lui, servait 12 M de tuiles par jour depuis le cache dans les racks, ce qui est cohérent avec 48 TB (une tuile fait 15 à 40 kB).

12 M de tuiles par jour pour 4 000 utilisateurs actifs du dispatch, c'est 3 000 tuiles par utilisateur et par jour. Une carte affiche 20 à 40 tuiles à la fois. Un dispatcher qui regarde la carte toute la journée et se déplace charge peut-être 500 tuiles nouvelles. Le reste était des rechargements de tuiles déjà vues.

## La cause

`Cache-Control: no-store` sur toutes les réponses du serveur de tuiles. Posé en mars 2024 (commit retrouvé) pour contourner un bug d'affichage où une tuile mise en cache par le navigateur avec un style obsolète restait affichée après un changement de style. Le correctif propre (un paramètre de version dans l'URL des tuiles, `?v=<hash du style>`) avait été fait deux mois plus tard, mais le `no-store` était resté. Depuis, chaque déplacement de carte, chaque zoom, chaque retour sur l'onglet rechargeait toutes les tuiles depuis le CDN, qui les servait depuis son cache (donc pas de charge sur nos racks, ce qui explique que personne côté serveur n'ait rien vu) et facturait l'egress à chaque fois.

Le CDN cachait bien (taux de succès 97 %). Le navigateur ne cachait rien. On payait 97 % du trafic pour resservir des tuiles que le navigateur avait eues dix secondes avant.

## La correction (HF-4710, 2025-11-20)

`Cache-Control: public, max-age=86400, immutable` sur les tuiles (l'URL porte déjà le hash du style, donc une tuile à une URL donnée ne change jamais), et `stale-while-revalidate=604800` pour que le navigateur continue d'afficher pendant qu'il revalide après un jour. Une ligne dans la configuration du serveur de tuiles, une relecture par la plateforme, déploiement un jeudi à 14:00.

## Mesuré

| | Semaine avant | Semaine après | Décembre 2025 |
|---|---|---|---|
| tuiles servies par le CDN par jour | 12,1 M | 3,4 M | 2,9 M |
| egress CDN par jour | 1,6 TB | 0,45 TB | 0,38 TB |
| egress facturé par mois (extrapolé puis réel) | 7 300 EUR | 2 200 EUR | 3 100 EUR (toutes origines, tuiles 1 100) |
| temps de rendu de la carte au chargement (p50, mesure côté client) | 1,4 s | 0,4 s | 0,4 s |
| tickets « la carte est lente » | 6 sur le mois | 1 | 0 |

Économie : 4 200 EUR par mois sur l'egress, soit 50 000 EUR par an, pour une ligne de configuration. La carte est aussi trois fois plus rapide à charger, ce qui était un sujet de plainte récurrent que personne n'avait relié au coût.

## Ce qu'on a vérifié après

- Que le bug de 2024 ne revient pas : un changement de style change le hash, donc l'URL, donc la tuile est une nouvelle ressource. Testé en staging avec un changement de couleur.

- Que les autres hôtes derrière le CDN n'ont pas le même défaut : `app.halden.example` avait des en-têtes corrects sur les ressources statiques ; l'API n'est pas derrière le CDN. Un seul hôte, une seule ligne.

- Que le rapport d'usage par hôte de Skyvale devienne une lecture mensuelle : il est maintenant dans la réconciliation ([[vendor-invoices-reconciliation]]) et une alerte d'anomalie ([[cost-anomaly-alerts]]) regarde l'egress par hôte par jour.

## Ce qu'on retient

Le coût était invisible parce qu'il n'y avait aucune douleur technique : le CDN absorbait tout, les racks ne voyaient rien, le serveur de tuiles était à 3 % de CPU. Le seul signal était la facture, et la facture était une ligne « egress » sans détail jusqu'à ce qu'on demande le rapport par hôte. La première chose que fait maintenant la personne qui saisit la facture Skyvale est d'ouvrir ce rapport ([[costs-reviewer-preferences]]).
