---
name: log-volume-finding-mobile-sync
description: A Loki volume breakdown in Jan 2026 showed 31 % of API log bytes were one info line per mobile sync pull with the full cursor and entity counts, cut to 4 % by sampling and moving counts to metrics
type: project
status: active
verified: 2026-02-09
---

# Finding: mobile sync was a third of the API's log volume

## How it was found

`hf-logs-volume` dashboard, panel "bytes per stream, top 20", built on `sum by (namespace, app) (bytes_over_time({...}[1h]))` in Loki, and for the API specifically a breakdown by the `channel` field of the JSON log line: `sum by (channel) (bytes_over_time({app="halden-api"} | json [1h]))`.

In the week of 2026-01-12, `channel="mobile_sync"` was 31 % of the API's bytes, and the API was 38 % of everything. One channel, 12 % of the whole Loki ingestion.

## What the line was

One `info` line per `GET /internal/mobile/sync` request, written by `SyncController` since the delta sync shipped:

```
{"channel":"mobile_sync","level":"info","message":"sync pull","driver_id":"...","cursor":"eyJj...(120 chars)","loads":3,"stops":9,"documents":1,"messages":0,"tombstones":0,"duration_ms":41,"trace_id":"..."}
```

About 400 bytes, 70 requests per minute at peak and 26 000 silent pushes a day each producing one, so around 110 000 lines a day, 45 MB a day raw. Not huge by itself, but it was the largest single stream, it was never queried (checked the Loki query logs for a month: zero queries filtering on `channel="mobile_sync"`), and the counts in it were the kind of thing a metric answers better.

## What changed (HF-1505)

- The line is kept, at `debug` level, so it is off in production and available on one pod when needed.

- The counts became a histogram and counters: `hf_mobile_sync_entities_total{kind="loads"}`, `hf_mobile_sync_duration_seconds` by `result`. About 40 series. The mobile sync dashboard now shows entity throughput over time, which the log line could never have shown without a heavy query.

- An `info` line is still written for the interesting cases only: `invalid_cursor` responses (422), full resyncs, and pulls that return more than 500 entities (`has_more`). Roughly 300 lines a day.

- The cursor is not logged at all anymore; if needed it is in the request in the ingress access log at `debug`, and the trace id joins them.

Result: `channel="mobile_sync"` went from 31 % to 4 % of API bytes. Total Loki ingestion dropped 11 %.

## The general rule that came out of it

A log line that carries numbers per request is a metric wearing a log's clothes. If the question is "how many" or "how long", it is a metric. If the question is "what happened to this specific request", it is a log line, and it does not need the numbers because the trace id leads to the metric's labels anyway.

Same exercise applied afterwards to the ingress access log (health probes removed, 30 % of ingress bytes) and to the Hubble flows (verdict `FORWARDED` flows dropped from the export, only `DROPPED` and `ERROR` kept, 60 % of Hubble bytes). See [[loki-retention-and-volume]] for the current volume and [[log-labels-cardinality-rule]] for the label side of the same problem.
