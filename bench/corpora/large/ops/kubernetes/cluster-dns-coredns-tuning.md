---
name: cluster-dns-coredns-tuning
description: CoreDNS on hf-main runs 4 replicas plus NodeLocal DNSCache after a burst of 5 s DNS timeouts from the API pods in Dec 2025 caused by conntrack races on UDP, ndots set to 2 on platform pods, cache TTL 30 s
type: project
status: active
verified: 2026-01-16
---

# DNS du cluster : CoreDNS et NodeLocal DNSCache (HF-INFRA-388)

## Le problème (décembre 2025)

Les logs de l'API montraient par vagues des `Could not resolve host: hf-postgres-rw.platform-prod.svc.cluster.local` et des timeouts de 5 secondes sur les appels vers les fournisseurs externes. Environ 40 occurrences par heure aux heures de pointe, aucune la nuit. 5 secondes, c'est le timeout par défaut du resolver glibc avant de réessayer : une requête DNS UDP perdue.

Cause connue et documentée ailleurs : une course dans conntrack quand deux requêtes UDP (A et AAAA en parallèle, ce que fait glibc) sortent avec le même tuple source. Une des deux est jetée. Ça touche tous les clusters Kubernetes avec kube-proxy ou son équivalent, et Cilium en mode remplacement n'y échappe pas complètement pour le trafic DNS vers le Service ClusterIP.

Second facteur : `ndots: 5` par défaut. Un appel à `api.push-provider.example` produisait 5 requêtes (`api.push-provider.example.platform-prod.svc.cluster.local`, `.svc.cluster.local`, `.cluster.local`, `.hf.internal`, puis le nom absolu), soit 10 avec AAAA, avant d'obtenir une réponse. Multiplié par 45 pods d'API.

## Ce qui a été fait

1. **NodeLocal DNSCache** en DaemonSet sur tous les nœuds, adresse de lien `169.254.20.10`. Les pods parlent à ce cache local sur le nœud, en TCP vers CoreDNS pour les misses, ce qui supprime la course conntrack UDP. Kubelet configuré avec `cluster-dns: 169.254.20.10`.

2. **CoreDNS** passé de 2 à 4 réplicas, avec anti-affinité par nœud, et `cache 30` dans le Corefile (était 30 aussi, mais avec `prefetch 10 60s 15%` ajouté pour les noms chauds).

3. **`ndots: 2`** dans le `dnsConfig` des pods de la plateforme (via un patch Kustomize sur tous les Deployments). Les noms internes sont écrits en forme courte à deux composants (`hf-postgres-rw.platform-prod`) ou en absolu avec le point final dans les variables d'environnement. Les FQDN externes ont 2 points ou plus et partent directement en absolu.

4. **`single-request-reopen`** dans `resolv.conf` via `dnsConfig.options`, ceinture et bretelles : glibc sérialise A et AAAA.

5. Le Corefile a un bloc `hf.internal` qui forward vers les DNS du datacentre, et `log` activé uniquement pour les réponses `SERVFAIL` et `NXDOMAIN` sur les zones internes (pas pour tout, voir la note d'observabilité sur le volume de logs).

## Mesures

- Timeouts DNS de 5 s dans les logs de l'API : 40 par heure → 0 sur les 4 semaines suivantes.

- Requêtes vers CoreDNS : 2 400 par seconde → 600 (le cache local absorbe le reste).

- Latence de résolution p99 depuis un pod : 12 ms → 0,4 ms (mesuré par une sonde qui résout un nom interne toutes les 10 s depuis chaque nœud).

- CPU CoreDNS : divisé par 3.

## Pièges rencontrés

- NodeLocal DNSCache avec Cilium en mode remplacement de kube-proxy demande le mode `bind` sur l'adresse de lien sans l'IP du Service `kube-dns` en plus, sinon les deux se battent. La doc du DaemonSet le dit dans une note de bas de page.

- Les pods créés avant le changement de `cluster-dns` de kubelet gardent l'ancien `resolv.conf`. Il a fallu un rollout de tout.

- Le `ndots: 2` casse un nom interne écrit avec un seul composant (`hf-postgres-rw` sans namespace). Il n'y en avait que deux, dans des CronJobs, corrigés.

## Surveillance

`coredns_dns_request_duration_seconds` p99 et `coredns_forward_healthcheck_failures_total` sur le dashboard DNS. Alerte `warn` si le taux de `SERVFAIL` dépasse 1 % pendant 10 minutes. Voir [[rke2-cluster-layout]] pour la place de CoreDNS sur les nœuds de contrôle.

## Corefile actuel et vérification depuis un pod

Le Corefile de CoreDNS tel qu'il est dans `clusters/hf-main/coredns/configmap.yaml` :

```
.:53 {
    errors
    health { lameduck 5s }
    ready
    kubernetes cluster.local in-addr.arpa ip6.arpa {
        pods insecure
        fallthrough in-addr.arpa ip6.arpa
        ttl 30
    }
    prometheus :9153
    forward . /etc/resolv.conf { max_concurrent 1000 }
    cache 30 { prefetch 10 60s 15% }
    loop
    reload
    loadbalance
}
hf.internal:53 {
    errors
    forward . 10.20.0.2 10.20.0.3
    cache 300
    log . { class denial error }
}
```

`lameduck 5s` fait que CoreDNS continue de répondre 5 s après avoir reçu `SIGTERM`, le temps que le Service retire le pod, sinon chaque rollout de CoreDNS produisait une salve de timeouts. Le `log` sur la zone interne ne garde que les refus et les erreurs, ce qui fait 30 lignes par jour au lieu de 600 000.

Le NodeLocal DNSCache a sa propre configuration, plus simple : forward de tout vers CoreDNS en TCP (`force_tcp`), cache 30 s, et le bloc `cluster.local` avec `prefer_udp` désactivé.

Vérifier depuis un pod, ce que l'astreinte fait en premier quand "ça ne résout pas" :

```
kubectl -n platform-prod exec deploy/halden-api -- cat /etc/resolv.conf
kubectl -n platform-prod exec deploy/halden-api -- getent hosts hf-postgres-rw.platform-prod
kubectl -n platform-prod exec deploy/halden-api -- sh -c 'time getent hosts api.push-provider.example'
```

Le `resolv.conf` doit montrer `nameserver 169.254.20.10`, `options ndots:2 single-request-reopen`. S'il montre `10.43.0.10` (l'IP du Service `kube-dns`), le pod a été créé avant la migration et un rollout le corrige. Un `time` au-dessus de 100 ms sur un nom externe la première fois puis sous 5 ms la seconde est normal (cache), un `time` de 5 s est le timeout UDP et ne devrait plus jamais arriver.

La sonde qui mesure la latence de résolution depuis chaque nœud (un DaemonSet minuscule qui fait `getent hosts` toutes les 10 s sur un nom interne et un nom externe et expose le résultat) a été gardée après la migration. Elle a une alerte `warn` à 100 ms de médiane sur 5 minutes, jamais déclenchée depuis janvier, et c'est la preuve que le problème est résolu et non pas déplacé.
