---
name: blackbox-probes-external
description: Two blackbox exporters, one inside ops-tools and one on a small VM outside the datacentre, probe every public hostname every minute for HTTP 200, TLS expiry and response time, and the outside one is the only source for is-the-site-up alerts
type: reference
status: active
verified: 2026-01-30
---

# Blackbox probes

## Two vantage points

- **Inside**: blackbox exporter on `ops-tools`, scraped by the central Prometheus. Sees the services through the same path as the ingress VIP. Good for latency trends and TLS checks, useless to know whether the internet can reach us.

- **Outside**: a small VM at a different provider, in another city, running a blackbox exporter and a minimal Prometheus agent that remote-writes to the central one over the VPN, and, when the VPN is down, buffers 6 hours. Its only job is to answer "can a client outside reach `api.halden.example`". The alert `SiteUnreachableExternal` fires only from this vantage point, `page`, `for: 3m`.

The outside VM is the one piece of infrastructure that is not in the racks and not in `hf-main`. It is rebuilt from a 40-line cloud-init in `halden-infra/external-probe/`, and its cost is a coffee a month.

## Targets

`Probe` objects (Prometheus operator CRD) in `clusters/ops-tools/monitoring/probes/`:

| Target | Module | Expect |
|---|---|---|
| `https://api.halden.example/health` | `http_2xx` | 200 with body `{"status":"ok"}` |
| `https://app.halden.example/` | `http_2xx` | 200 and body contains `<div id="root">` |
| `https://live.halden.example/healthz` | `http_2xx` | 200 |
| `https://tiles.halden.example/health` | `http_2xx` | 200 |
| `https://api.staging.halden.example/health` | `http_2xx` | 200 (warn only) |
| `https://registry.hf.internal/v2/` (inside only) | `http_2xx_internal_ca` | 200 or 401 |
| `10.20.0.50:443` (inside only) | `tcp_connect` | the VIP answers |

Interval 60 s, timeout 10 s. The `http_2xx` module follows redirects (`follow_redirects: true`), uses IPv4 only (no IPv6 on our public address), and checks `fail_if_ssl: false`, `fail_if_not_ssl: true` for the public hosts.

## What each probe yields

- `probe_success`: the boolean everything else derives from.

- `probe_duration_seconds` and the phase breakdown (`probe_http_duration_seconds{phase="connect|tls|processing|transfer"}`). The `tls` phase from outside is 90 ms, from inside 8 ms, which is the handshake round trip and a useful sanity number.

- `probe_ssl_earliest_cert_expiry`: the certificate actually served, independent of cert-manager's opinion. This is the alert that would have caught [[incident-2025-10-ingress-cert-expired]] three weeks early, and it is in the page list of [[alerting-rules-catalogue]] at 21 days.

- `probe_http_status_code`, `probe_http_version`, `probe_http_content_length` for the dashboard.

## Alerts

- `SiteUnreachableExternal` (page): `probe_success{vantage="external", target=~"api|app|live"} == 0` for 3 min. Three minutes rather than one because the outside VM's link has had 60 s blips.

- `SiteUnreachableInternal` (warn): same from inside. If the internal one fires and the external does not, it is our monitoring path, not the site.

- `CertificateExpiringSoon` (page): described above.

- `ProbeLatencyHigh` (warn): `probe_duration_seconds{target="api"} > 1.5` for 10 min from outside.

- `ExternalProbeSilent` (page): `absent_over_time(up{vantage="external"}[15m])`. If the outside VM stops reporting, we want to know, because otherwise its absence looks like "everything is fine".

## Things that surprised us

- The `app` probe once failed for 20 minutes with 200 responses: the body check caught a deploy that served a maintenance page with status 200. That body check is not decoration.

- A probe from inside through the VIP sees the ingress but not the datacentre firewall's NAT. The two vantage points disagreed for a full afternoon in 2025 during a firewall rule change, and the outside one was right.

- `follow_redirects` plus the HSTS redirect on `http://` meant the first probes were checking the redirect, not the app. The targets are all `https://` now.
