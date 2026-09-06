---
name: tracing-otel-collector
description: Distributed tracing is partial, the API emits OpenTelemetry spans (PHP SDK, 5 % head sampling plus 100 % for errors and slow requests) through an OTel collector on hf-main to a Tempo instance on ops-tools with 7 d retention, the front and mobile only propagate traceparent
type: project
status: active
verified: 2026-05-07
---

# Tracing: what exists and what does not

Started in HF-1550 (March 2026), deliberately limited in scope. The `trace_id` in logs and error envelopes predates this and comes from the same `traceparent` header.

## Pipeline

- **API**: the OpenTelemetry PHP SDK with auto-instrumentation for Symfony HTTP, Doctrine DBAL, the HTTP client and Messenger. Exporter OTLP over gRPC to the collector's service. Spans carry `hf.route`, `hf.org_id`, `db.statement` (truncated at 500 chars, parameters never included), `messenger.message_class`.

- **Collector**: OTel collector as a Deployment (2 replicas) in `ops` on `hf-main`, receivers OTLP, processors `memory_limiter`, `batch`, `tail_sampling` (below), exporter OTLP to Tempo on `ops-tools`.

- **Storage**: Tempo (monolithic mode, 1 replica, object store backend, bucket `hf-tempo`), 7 days retention. Queried from Grafana with the Tempo data source, linked from Loki (a `trace_id` in a log line is a clickable link) and from the API dashboard (exemplars on the latency histogram).

## Sampling

Head sampling at 5 % in the PHP SDK (`OTEL_TRACES_SAMPLER=parentbased_traceidratio`, `OTEL_TRACES_SAMPLER_ARG=0.05`), so the SDK cost stays around 1 % CPU on the API pods. Then tail sampling in the collector keeps:

- 100 % of traces with an error span

- 100 % of traces longer than 1 s

- 100 % of traces from the `support` role (they are the ones investigating)

- 10 % of the rest

Combined, about 0.7 % of requests end up stored, roughly 60 000 traces a day, 4 GB a day in Tempo.

The catch with head sampling: a request that the API decides not to sample cannot be recovered by the tail sampler, because there are no spans. So "100 % of errors" really means "100 % of errors among the 5 % head-sampled, plus every error the API forces": the SDK is configured to force-sample when the response is 5xx or when `X-Halden-Debug: 1` is set by a support user. This is done in `TraceSamplingSubscriber` on `kernel.response`... which is too late to affect the root span's sampling decision, so in practice the root span is always recorded and the decision is applied at export time in a custom span processor. Written down because two people rediscovered it.

## What propagates and what does not

- The web front sends `traceparent` on every request (generated in `http.ts`), so a support investigation can start from a `trace_id` copied from the toast. It does not emit spans: browser-side tracing was tried and produced more data than value.

- The driver app sends `traceparent` per request too, with a trace id derived from the mutation id for outbox pushes, so retries of the same mutation share a trace. No spans either.

- Workers continue the trace of the HTTP request that dispatched the message through the Messenger stamp, so an invoice PDF generation appears under the request that triggered it.

- The outbox relay starts a new trace per delivery and links it to the originating trace with a span link.

## What it has been useful for

- The February search timeout incident analysis: the traces showed the `Seq Scan` duration inside the request before anyone ran `EXPLAIN`. Actually the incident predates the pipeline; the same analysis on a later slow query is what it was useful for.

- Finding that `GET /v2/loads/{id}` made two identical `carrier_scores` queries per request (a normaliser and a voter both loading it), 4 ms saved per call, HF-1575.

- Support answers "why was this request slow" from a trace id in about a minute.

## What is missing

Spans from the live gateway and the routing service (planned), a second Tempo replica, and a sampling rule per route (the search route deserves more than 5 %). See [[prometheus-stack-layout]] for the metrics side and [[loki-retention-and-volume]] for logs; the three are linked by `trace_id` and that link is worth more than any of them alone.

## Cost and sizing

- The PHP SDK adds about 1 % CPU on the API pods at 5 % head sampling, measured by comparing two pods with the sampler at 0 and at 0.05 over a day. At 100 % it was 9 %, which is why head sampling exists at all.

- The collector pods run at 200 to 400 millicores and 600 MB each. The `tail_sampling` processor holds every trace for `decision_wait: 10s` before deciding, and at peak that is 12 000 open traces in memory. `memory_limiter` is set to 1 GB with a 200 MB spike limit, and when it triggers the collector refuses new spans for a few seconds, which shows up as `otelcol_processor_refused_spans` on the dashboard. It has triggered twice, both during the sync storm in December (traces with hundreds of spans from the retry loop).

- A stored trace is on average 9 KB in Tempo's compressed blocks, 60 000 traces a day is 4 GB a day before compaction and about 2.5 GB after, and 7 days of retention keeps the bucket around 20 GB.

- Tempo's query path: `max_bytes_per_trace: 5MB` (one export trace with 30 000 spans hit this and was dropped, which is correct), search limited to the last 7 days by definition, and a `TraceQL` query over a day on a route attribute takes 2 to 4 s. Exemplars from the API's latency histogram link straight to a trace id, which avoids searching at all in the common case.

## Attributes we forbid on spans

The same cardinality logic as metrics does not apply (Tempo does not index by attribute value the way Prometheus does), but two rules exist for other reasons: no personal data in attributes (`db.statement` is truncated and parameter-free, HTTP headers are not recorded, the user id is the org id and role only), and no attribute above 1 KB (a `messenger.payload` attribute was added once for debugging and doubled the trace size). The span processor that enforces the size limit also strips any attribute whose key starts with `hf.debug.` outside the `dev` environment, so developers can add temporary attributes without a cleanup PR.

## What a support investigation looks like now

The dispatcher's toast shows `Code support : 0a1f3c9e`. Support pastes the full trace id (from the copy button) into Grafana's Explore with the Tempo source, sees the request's spans (the Doctrine queries with their durations, the Messenger dispatch, the outbox insert), and if the request was slow, which span. If the trace is not there (not sampled, and not forced because it was a 200 under 1 s), the same id in Loki still gives the log lines. That two-step is documented in the support handbook and takes about a minute, against the 10 to 15 minutes of asking an engineer before.
