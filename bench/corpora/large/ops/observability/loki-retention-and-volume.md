---
name: loki-retention-and-volume
description: Loki in simple scalable mode on ops-tools with object store chunks, 14 d retention by default (90 d for audit, 3 d for Hubble flows), ingests 95 GB/day compressed to 12 GB, and the per-stream limits that keep it alive
type: project
status: active
verified: 2026-04-24
---

# Loki : rétention et volume

## Déploiement

Mode "simple scalable" sur `ops-tools` : 2 `write`, 3 `read`, 1 `backend`. Chunks et index dans le magasin d'objets (bucket `hf-loki`), WAL des writers sur Longhorn (20 Go chacun). Agent de collecte : un DaemonSet par cluster qui lit les logs des conteneurs et le journal système, avec les pipelines décrits dans [[log-labels-cardinality-rule]].

## Rétention par flux

Configurée dans le compactor avec `retention_stream` :

| Sélecteur | Rétention | Pourquoi |
|---|---|---|
| défaut | 14 jours | assez pour une revue d'incident |
| `{namespace="kube-system", app="kube-apiserver-audit"}` | 90 jours | exigence du questionnaire sécurité |
| `{source="hubble"}` | 3 jours | volumineux, utile seulement pour déboguer une politique réseau la semaine même |
| `{namespace="platform-prod", app="halden-api", level="error"}` | 60 jours | les erreurs applicatives servent aux post-mortems tardifs |
| `{namespace="platform-staging"}` | 3 jours | personne ne relit le staging |

La suppression est effective sous 24 h après la fin de la rétention (`retention_delete_delay: 2h`, compaction toutes les 10 minutes).

## Volume (avril 2026)

- Ingestion : 95 Go par jour bruts, environ 12 Go par jour stockés après compression (chunks en `snappy`).

- Répartition : API et workers 38 %, Hubble 22 %, ingress 17 %, système Kubernetes et audit 10 %, télémétrie mobile 8 %, reste 5 %.

- Stockage total dans le bucket : 240 Go.

- Débit de requête typique : 2 à 5 requêtes par seconde, pics à 40 pendant un incident. Les 3 readers suffisent, avec un cache de résultats de 2 Go en mémoire.

Le poste ingress a baissé de 30 % après avoir retiré les lignes d'accès des sondes de santé (`/health` toutes les 5 s depuis 45 pods, voir [[log-volume-finding-mobile-sync]] pour la méthode qui l'a trouvé).

## Limites qui protègent Loki

Dans `limits_config` :

- `ingestion_rate_mb: 20`, `ingestion_burst_size_mb: 40` par tenant (on a un seul tenant, `hf`).

- `per_stream_rate_limit: 5MB`, `per_stream_rate_limit_burst: 15MB`. Un pod qui boucle sur une erreur produit un flux à 50 Mo/s, et sans cette limite il fait tomber les writers pour tout le monde. Avec, ses lignes sont rejetées avec un `429` côté agent, qui les jette après 3 tentatives, et une métrique `loki_discarded_samples_total{reason="per_stream_rate_limit"}` monte, ce qui alerte en `warn` avec le nom du flux. C'est arrivé quatre fois, à chaque fois un worker d'export en boucle sur un document corrompu.

- `max_streams_per_user: 50000`. On est à 18 000. Dépasser, c'est presque toujours un label à forte cardinalité qui vient d'être ajouté, voir [[log-labels-cardinality-rule]].

- `max_line_size: 256KB`, les lignes plus longues sont tronquées (des dumps de payload dans les logs d'erreur PHP).

- `max_query_length: 30d`, `max_query_parallelism: 16`, `query_timeout: 2m`.

## Ce qu'on ne loggue pas

- Le corps des requêtes et des réponses HTTP. Jamais.

- Les logs d'accès des sondes de santé et de métriques (filtrés par l'agent sur `path=/health` et `path=/metrics`).

- Les logs `debug` en production. `APP_LOG_LEVEL=info` sur l'API, et le passage en `debug` à chaud se fait par variable sur un seul pod pendant 10 minutes maximum, avec un rappel automatique.

## Requêtes utiles

- Une requête par `trace_id` (le label est indexé, voir [[log-labels-cardinality-rule]]) : `{namespace="platform-prod"} | json | trace_id="0a1f3c9e8b7d4e2f"`, qui traverse l'ingress, l'API et les workers.

- Le taux d'erreurs par route : `sum by (route) (rate({app="halden-api"} | json | level="error" [5m]))`, mais pour ça les métriques sont mieux.

## Ce qui reste à faire

Passer le WAL des writers de Longhorn au NVMe local (les writers sont sur des nœuds fixes de `ops-tools`), pour la même raison que PostgreSQL. Et une seconde réplique du `backend` pour que la compaction survive à un redémarrage de nœud.

## Le chemin de requête et ses caches

Les `read` pods reçoivent les requêtes via le query frontend intégré, qui découpe une requête sur 14 jours en 14 requêtes d'un jour exécutées en parallèle (`split_queries_by_interval: 24h`, `max_query_parallelism: 16`). Deux caches en mémoire dans les readers : le cache de résultats (2 Go, clé = requête + intervalle, TTL 1 h, taux de succès 60 % parce que les dashboards répètent les mêmes requêtes) et le cache de chunks (4 Go, taux de succès 85 % sur les 24 dernières heures, presque nul au-delà, ce qui est attendu).

Les trois requêtes lentes identifiées par le journal de requêtes de Loki (`query_stats` dans les logs des readers), et ce qu'on en a fait :

- Le panneau "erreurs par route" du dashboard API, qui parsait le JSON de toutes les lignes `error` sur 24 h : 9 s. Remplacé par la métrique `hf_http_requests_total{status=~"5.."}` par route, ce que ça aurait dû être depuis le début.

- La recherche d'un `trace_id` sur 14 jours par le support : 2 à 6 s, acceptée, et souvent réduite à 1 s depuis que `trace_id` est en structured metadata.

- Le décompte de lignes par pod pour le nettoyage des logs (`count_over_time` avec `| json | pod_name`) : 20 s sur 7 jours. Remplacé par un panneau sur `bytes_over_time` par `app`, qui suffit pour trouver les coupables, et l'analyse fine par pod se fait sur une heure.

`max_entries_limit_per_query: 5000` : une requête qui retournerait plus de 5 000 lignes est tronquée avec un avertissement dans Grafana. Personne ne lit 5 000 lignes, et avant cette limite un `{namespace="platform-prod"}` sans filtre sur une journée faisait tomber un reader par manque de mémoire. `max_query_series: 500` pour les requêtes métriques, pour la même raison.

Un point de vocabulaire qui a créé de la confusion : "rétention" dans Loki s'applique aux chunks dans le magasin d'objets, mais l'index (TSDB) a sa propre période, `index.period: 24h`, et les fichiers d'index de plus de 14 jours sont supprimés par le compactor en même temps que les chunks. Le WAL des writers sur Longhorn, lui, ne garde que ce qui n'est pas encore flushé, environ 30 minutes, et sa taille de 20 Go est très largement surdimensionnée, ce qu'on a gardé pour le cas d'un magasin d'objets indisponible pendant quelques heures (testé : 3 h de panne du magasin, aucune ligne perdue, le WAL était à 6 Go).
