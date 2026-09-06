---
name: feature-flags-mobile-remote-config
description: Mobile feature flags come from GET /internal/mobile/config (cached 10 min, bundled defaults in lib/config/defaults.dart), targeted by app version, platform, carrier id and a percentage on installation_id, no third-party remote config
type: reference
status: active
verified: 2026-03-24
---

# Feature flags dans l'app chauffeur

Pas de service tiers de remote config. Les flags viennent de notre API, table `sys_feature_flags` côté serveur, éditée depuis le back-office par les personnes qui ont le rôle `release_manager`.

## Lecture

`GET /internal/mobile/config` renvoie :

```json
{
  "flags": { "new_stop_screen": true, "pod_edge_detection": false },
  "values": { "sync_timer_minutes": 15, "photo_max_edge_px": 1600 },
  "ttl_seconds": 600
}
```

L'app appelle l'endpoint au démarrage et à chaque synchronisation, garde la réponse dans `sync_state.config_json` et la considère valide pendant `ttl_seconds`. Sans réseau, la dernière réponse connue sert, sans limite de durée. Sans réponse du tout (première installation hors ligne, ça arrive), les valeurs de `lib/config/defaults.dart` s'appliquent. Ce fichier est la source de vérité du "comportement sûr" : un flag y est toujours à la valeur la plus conservatrice.

`AppConfig.of(context).flag('new_stop_screen')` en lecture, jamais de lecture directe du JSON dans un widget.

## Ciblage côté serveur

Chaque flag a des règles évaluées dans l'ordre, la première qui correspond gagne :

- `app_version >= 4.8.0` (comparaison semver)

- `platform in [ios]`

- `carrier_id in [...]` (un transporteur pilote)

- `percentage: 20` sur un hash stable de `installation_id`, donc un appareil donné reste dans ou hors du lot d'une lecture à l'autre

- sinon la valeur par défaut du flag côté serveur

Le `installation_id` est envoyé en en-tête `X-Installation-Id` sur cet appel, c'est le seul endroit où il sert au ciblage.

## Règles d'usage

- Un flag a un ticket et une date de retrait prévue. Au-delà de 3 mois après le passage à 100 %, le flag est retiré du code et de la table. On en a eu 14 morts-vivants en 2025, plus jamais.

- Un flag ne change pas de sens entre deux versions. Si le comportement change, nouveau flag.

- Pas de flag pour du texte ou des couleurs, c'est de la config, pas un flag, ça va dans `values`.

- Les valeurs de `values` ont un type et une borne vérifiés à la lecture (`sync_timer_minutes` entre 1 et 60). Une valeur hors borne déclenche la valeur par défaut et un événement de télémétrie `config.invalid_value`.

## Lien avec les releases

Le processus de sortie ([[release-process-stores]]) exige que toute fonctionnalité qui touche la synchronisation, les photos ou le suivi GPS parte derrière un flag à `false`. Le passage à `true` se fait par paliers de transporteurs, pas par pourcentage, parce qu'un dispatcher qui voit deux comportements différents chez ses chauffeurs appelle le support.
