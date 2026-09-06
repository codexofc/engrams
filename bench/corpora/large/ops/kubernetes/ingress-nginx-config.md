---
name: ingress-nginx-config
description: ingress-nginx (controller 1.12) on two dedicated ingress nodes with a kube-vip VIP, global proxy-body-size 20m, timeouts 60 s (120 s for /internal/ws), long cache headers on hashed assets, and the per-User-Agent block snippet
type: reference
status: active
verified: 2026-04-05
---

# ingress-nginx configuration

Replaced Traefik in September 2025, see [[ingress-traefik-legacy]] for what was there before and why it went.

## Placement

Controller as a DaemonSet on the two nodes labelled `ingress=true` (`hf-wk-01` and `hf-wk-02`), `hostNetwork: true`, listening on 80 and 443. A kube-vip VIP (`10.20.0.50`) floats between them in ARP mode, and the datacentre firewall NATs the public address to it. Failover measured at 3 s when a node dies. The public hostnames (`api.halden.example`, `app.halden.example`, `api.staging.halden.example`, `tiles.halden.example`, `live.halden.example`) all resolve to the same public address.

## Global settings (ConfigMap `ingress-nginx-controller`)

- `proxy-body-size: 20m` (documents go direct to object storage with presigned URLs, so the API never receives a large body; 20m covers the biggest JSON payloads with margin).

- `proxy-read-timeout: 60`, `proxy-send-timeout: 60`. The API's own `statement_timeout` is 8 s, so 60 s is never the limiting factor except on exports, which have their own annotation at 180.

- `use-forwarded-headers: false`, we are the edge, nothing in front sets those headers, and trusting them would let a client spoof its IP for the rate limiter.

- `enable-real-ip: true` with `proxy-real-ip-cidr: 10.20.0.0/24`.

- `ssl-protocols: TLSv1.2 TLSv1.3`, `ssl-ciphers` set to the intermediate profile. TLS 1.2 stays because two carrier integrations run on old Java runtimes.

- `hsts: true`, `hsts-max-age: 31536000`, `hsts-include-subdomains: false` (the tile server has a separate cert story).

- `log-format-upstream` in JSON with `trace_id` from the `traceparent` request header, so the ingress log line and the API log line join on it in Loki.

- `worker-processes: 8`, `max-worker-connections: 65536`.

## Per-ingress annotations we use

- `nginx.ingress.kubernetes.io/proxy-read-timeout: "120"` on the `/internal/ws` path for the live gateway (heartbeat is 25 s, server closes at 60 s idle, ingress must be above both).

- `nginx.ingress.kubernetes.io/affinity: cookie` on the live gateway so a reconnect goes back to the same replica when possible (replay buffers are per replica).

- `nginx.ingress.kubernetes.io/configuration-snippet` with `more_set_headers "Cache-Control: public, max-age=31536000, immutable";` on the web front's `/assets/` location, and `no-cache` on `/index.html`.

- `nginx.ingress.kubernetes.io/limit-rps` is **not** used for the API (the application limiter knows the organisation, the ingress does not). It is used on `/v2/auth/*` at 20 rps per IP as a first layer, see [[ingress-rate-limiting-waf]].

## The `block-mobile-version` snippet

Kept in `components/ingress/snippets/block-mobile-version.yaml`, applied by uncommenting it in the API ingress overlay:

```
if ($http_user_agent ~* "HaldenDriver/4\.5\.2") { return 403; }
```

Used during the December 2025 sync storm and once in March 2026. The point of keeping it in the repo is not to write nginx conditionals under stress.

## Certificates

Issued by cert-manager, see [[cert-manager-and-letsencrypt]]. The default certificate for unknown hosts is a self-signed one so that scanners hitting the IP by address do not get a real hostname in the certificate.

## Metrics

The controller exposes `nginx_ingress_controller_requests` and latency histograms, scraped by the ops Prometheus. Dashboards per host and per ingress. The alert `IngressUpstream5xxHigh` fires on 5xx rate over 5 % for 5 minutes per ingress, `warn` only, because the application-level alerts page first.

## Known limitations

- A configuration reload (every ingress change) drops long-lived WebSocket connections held by the old worker after the `worker-shutdown-timeout` (240 s). Ingress changes during peak hours cause a wave of reconnects on the live gateway. Ingress changes are batched and applied at 13:00 or after 18:00.

- `hostNetwork` means only one controller per node, and the two ingress nodes cannot run a second ingress class. Fine so far.

## Reload behaviour, the 13:00 rule, and header hygiene

Every change to an Ingress object, a referenced Secret, or the controller ConfigMap makes the controller regenerate `nginx.conf` and reload. A reload starts new worker processes and lets the old ones finish their connections for up to `worker-shutdown-timeout` (240 s), then kills them. HTTP requests never notice. WebSocket connections live on the old workers until the kill, so 240 s after any ingress change, every live gateway socket that was open before the change is closed at once, and 700 clients reconnect within 3 s. It is handled (replay from `seq`), it costs a burst on the ticket endpoint, and it is why ingress changes are applied at 13:00 or after 18:00, when about 200 dispatchers are connected instead of 700.

The controller's dynamic configuration (Lua) handles endpoint changes without a reload, so pod rollouts of the API do not trigger one. `nginx_ingress_controller_success` (reload count) and `nginx_ingress_controller_config_last_reload_successful` are on the ingress dashboard, and a reload count above 5 per hour is a `warn`, because it once revealed a CronJob that patched an Ingress annotation every minute.

Headers added at the ingress for the web front, through a `configuration-snippet` on the `halden-web` Ingress, so that the front's nginx image stays a dumb static server:

- `Content-Security-Policy: default-src 'self'; connect-src 'self' https://api.halden.example wss://live.halden.example https://tiles.halden.example; img-src 'self' data: https://tiles.halden.example; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'`. The `'unsafe-inline'` for styles is the map library's inline style attributes, and removing it is HF-1595, not scheduled.

- `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy: geolocation=(), camera=()`.

- `X-Frame-Options` is not set; `frame-ancestors 'none'` in the CSP covers it and the duplication confused a security scanner once.

For the API host, only `X-Content-Type-Options` and `Cache-Control: no-store` are added; the API sets everything else itself, including CORS (`Access-Control-Allow-Origin` restricted to the two front hostnames, handled by the Symfony CORS listener rather than the ingress so that the allowed origins list lives with the application).

`server-tokens: false` in the controller ConfigMap removes the nginx version from error pages and headers, and the default backend (what answers when no Ingress matches the host) is a tiny image that returns a plain 404 with no body, so that scanners probing the IP by address learn nothing.
