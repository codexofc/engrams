---
name: hpa-and-resource-requests
description: Every platform pod has CPU and memory requests (admission enforced), limits on memory only, HPA on the API from 12 to 60 pods on CPU 65 % and on php-fpm active workers, worker deployments scaled on RabbitMQ queue depth via KEDA
type: project
status: active
verified: 2026-02-20
---

# Requests, limits et autoscaling

## Règles sur les ressources

- **Requests obligatoires** en CPU et mémoire sur tout conteneur. La politique d'admission refuse un pod sans. Sans request, le scheduler place à l'aveugle et le nœud se retrouve à 120 % de mémoire.

- **Limite mémoire obligatoire**, égale à la request ou au plus 1,5× (`QoS Burstable`). Un pod qui dépasse est tué proprement, ce qui vaut mieux que l'éviction d'un voisin.

- **Pas de limite CPU** sur les pods applicatifs. Le throttling CFS produisait des latences p99 à 300 ms sur l'API à cause de pics courts, mesuré en 2025 ; sans limite, un pod peut emprunter le CPU inutilisé du nœud et le scheduler reste guidé par les requests. La seule exception est les workers d'export (limite 4 CPU) parce qu'un export mal écrit a un jour pris 30 cœurs.

- Les valeurs de request viennent des mesures (`container_cpu_usage_seconds_total` p95 sur 7 jours, `container_memory_working_set_bytes` p99), revues chaque trimestre par un script qui propose un diff.

Valeurs actuelles (prod) :

| Composant | CPU request | Mémoire request / limite |
|---|---|---|
| `halden-api` (php-fpm, 12 workers) | 1500m | 900Mi / 1200Mi |
| `halden-api-worker-async` | 500m | 400Mi / 512Mi |
| `halden-api-worker-exports` | 1000m | 1Gi / 1536Mi |
| `halden-web` (nginx statique) | 50m | 64Mi / 96Mi |
| `live-gw` | 300m | 256Mi / 384Mi |
| `pgbouncer-api` | 500m | 128Mi / 192Mi |

## HPA sur l'API

```yaml
minReplicas: 12
maxReplicas: 60
metrics:
  - type: Resource
    resource: { name: cpu, target: { type: Utilization, averageUtilization: 65 } }
  - type: Pods
    pods:
      metric: { name: phpfpm_active_processes_ratio }
      target: { type: AverageValue, averageValue: "0.7" }
behavior:
  scaleUp:   { stabilizationWindowSeconds: 30,  policies: [{ type: Pods, value: 8, periodSeconds: 60 }] }
  scaleDown: { stabilizationWindowSeconds: 600, policies: [{ type: Percent, value: 10, periodSeconds: 60 }] }
```

La seconde métrique (ratio de workers php-fpm actifs, via l'adaptateur Prometheus) est celle qui réagit : le CPU monte tard quand les workers attendent la base. Montée rapide (8 pods par minute), descente lente (10 % par minute après 10 minutes stables) parce que le pic de 7 h 30 a des creux de 2 minutes qui faisaient osciller la première version.

Observé : 12 pods la nuit, 45 à 7 h 30 le lundi, 28 en journée. Le maximum de 60 n'a jamais été atteint ; s'il l'était, la base serait le goulot avant les pods (voir la note PgBouncer côté API).

## Workers : KEDA sur la profondeur des files

Les Deployments `halden-api-worker-*` sont scalés par KEDA (`ScaledObject`, déclencheur `rabbitmq`) :

- `async` : 1 pod par 200 messages en attente, min 2, max 12.

- `async_priority` : 1 par 50, min 2, max 8.

- `notifications` : 1 par 100, min 1, max 6.

- `exports` : 1 par 5, min 1, max 4 (limité par la mémoire).

`cooldownPeriod: 300` pour ne pas tuer un worker au milieu d'un message long, en plus du `terminationGracePeriodSeconds: 90` du Deployment.

## Ce qu'on ne scale pas automatiquement

`live-gw` (2 réplicas fixes, les reconnexions massives coûtent plus que le CPU économisé), `pgbouncer-api` (2 fixes, la taille du pool est la vraie limite), le relais outbox (1 fixe, par conception). Voir [[kubernetes-lesson-pdb-everywhere]] pour ce qui accompagne tout Deployment à plusieurs réplicas.

## La revue trimestrielle des requests

`scripts/rightsize.py` dans `halden-infra`, lancé à la main au début de chaque trimestre, fait pour chaque conteneur de `platform-prod` :

1. `quantile_over_time(0.95, rate(container_cpu_usage_seconds_total[5m])[7d:5m])` par conteneur, puis la médiane sur les pods du même Deployment.

2. `max_over_time(container_memory_working_set_bytes[7d])` par conteneur, p99 sur les pods.

3. Compare avec les `requests` du manifeste (lues depuis le rendu Kustomize), et propose une nouvelle valeur : CPU = p95 × 1,3 arrondi aux 50 m, mémoire = p99 × 1,2 arrondi aux 32 Mi, limite mémoire = request × 1,3.

4. Sort un diff YAML prêt à coller dans l'overlay, avec pour chaque ligne l'ancienne valeur, la mesure et la nouvelle valeur.

Le résultat n'est jamais appliqué tel quel. Ce qui a été retenu à la revue d'avril 2026 :

- `halden-api` : CPU request de 1500 m à 1200 m (p95 mesuré 850 m). Gain : 45 pods × 300 m = 13,5 cœurs de requests libérés, ce qui a repoussé l'achat du huitième nœud d'un trimestre. Mémoire inchangée.

- `halden-api-worker-exports` : mémoire request de 1 Gi à 1,25 Gi, parce que le p99 était à 1,1 Gi et qu'on a eu deux OOM sur des exports de 50 000 lignes. La limite est passée à 1,75 Gi.

- `live-gw` : CPU request de 300 m à 150 m. Le script proposait 100 m, on a gardé une marge pour les reconnexions massives.

- `pgbouncer-api` : inchangé, le script propose toujours moins et on sait que la charge est en pointes courtes que le p95 sur 5 minutes ne voit pas.

Le script ignore volontairement les CronJobs (trop courts pour des percentiles utiles) et les pods de moins de 7 jours d'existence. Il a un mode `--check` utilisé en CI qui échoue si un conteneur a une request CPU plus de 3× au-dessus de son p95 mesuré, en `warn` seulement : c'est un rappel, pas un blocage, parce que la mesure d'un trimestre calme ne dit rien du lundi de rentrée.

Ce qu'on a appris en trois trimestres : les requests dérivent vers le haut si personne ne les regarde (chaque incident fait monter une valeur et rien ne la redescend), et le seul moyen de les faire redescendre est un chiffre mesuré posé à côté de la valeur dans une MR que quelqu'un doit approuver.
