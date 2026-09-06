---
name: websocket-live-updates
description: The board gets live updates over one WebSocket per tab (endpoint /internal/ws, gateway service live-gw fed by the API outbox), messages are invalidation hints not data, with a reconnect and resync protocol, 50 updates/min per board in peak
type: project
status: active
verified: 2026-05-19
---

# Live updates on the dispatch board (HF-1170, revised HF-1520)

## Why invalidation hints and not data

The first version (2024) pushed full load objects over the socket and wrote them into the store. It raced with in-flight GETs: a slow list response arriving after a socket update overwrote the newer state. With TanStack Query (see [[dispatch-board-state-zustand]]) the design became simpler: the socket only says *what changed*, and the query cache refetches. The server never has to know what the client displays.

Message shape:

```json
{ "v": 1, "kind": "load.changed", "id": "8a4f2c17-...", "at": "2026-05-12T09:14:22.184Z", "seq": 8812 }
```

`kind` in `load.changed`, `load.removed`, `bid.changed`, `message.new`, `carrier.position` (the only one that carries data: lat, lng, at, because refetching a position per truck per 10 s would defeat the purpose).

## Server side

A small Node service `live-gw` (not part of the PHP API) subscribes to a RabbitMQ exchange `hf.live` that the API's outbox relay publishes to for every domain event, in addition to the external webhooks. `live-gw` keeps a map of connected sockets by organisation and forwards events to the sockets of the shipper or carrier concerned. It has no database, no state beyond the connection map, and 2 replicas behind the ingress with sticky sessions on a cookie. A replica restart drops its sockets, clients reconnect within 3 s to the other.

Auth: the client opens the socket with a short-lived ticket obtained from `POST /internal/ws/ticket` (valid 30 s, single use), not with the JWT in the URL, so the token never appears in access logs.

## Client side (`src/live/LiveClient.ts`)

- One socket per tab. A `BroadcastChannel` election makes one tab the leader that holds the socket and rebroadcasts events to the others. Before this, a dispatcher with 6 tabs had 6 sockets and `live-gw` counted 4 000 connections for 700 users.

- On `load.changed`: `queryClient.invalidateQueries({ queryKey: ['loads', 'detail', id] })` and `['loads', 'list']`. The list invalidation is debounced 500 ms, because a bulk assignment produces 40 events in a second and 40 list refetches would be silly.

- On `carrier.position`: `queryClient.setQueryData(['positions', carrierId], ...)` directly, no refetch.

- `seq` is monotonic per organisation. The client stores the last seen `seq`. On reconnect it sends `{ "resume": 8812 }`; `live-gw` keeps the last 500 events per organisation in memory for 10 minutes and replays the gap. If the gap is larger or older, it answers `{ "resync": true }` and the client invalidates everything (`queryClient.invalidateQueries()`), which is a full board reload in about 1 s.

- Heartbeat: ping every 25 s from the client, the server closes a socket silent for 60 s. The ingress idle timeout is 120 s, above both.

## Reconnect

Exponential backoff from 1 s to 30 s with jitter, and no reconnect attempt while `document.visibilityState === 'hidden'` (a hidden tab that reconnects every 30 s all night is pointless). On visibility change to visible, reconnect immediately and resume.

## Numbers (May 2026)

- Connected sockets at peak: 720 (one per leader tab).

- Events forwarded per minute at peak: 36 000 in total, about 50 per board.

- `live-gw` memory: 180 MB per replica, mostly the replay buffers.

- End-to-end latency, API commit to board refetch complete: p50 400 ms, p95 1.4 s.

## Failure modes seen

- The BroadcastChannel leader tab is closed: the election takes up to 2 s, during which events are lost. Covered by the resume protocol since the new leader resumes from the shared last `seq` (stored in `localStorage`).

- `live-gw` deploy during peak: 700 reconnects in 3 s, and 700 resume requests. Fine, replay is from memory. But 700 `/internal/ws/ticket` requests hit the PHP API in the same 3 s, which showed as a small latency bump. Tickets are now requested with a random delay of 0 to 2 s on reconnect (not on first connect).

- A dispatcher's laptop sleeping and waking: the socket is dead but the browser does not know for up to 60 s. The heartbeat handles it, and `visibilitychange` triggers an immediate ping.

## What it does not do

No delivery guarantee beyond the 10 minute replay. The board is always correct after a refetch, the socket only makes it fresh sooner. If `live-gw` is down entirely, the board falls back to the 30 s `staleTime` and focus refetch, and a small "Mises à jour en direct indisponibles" badge shows in the header. Dispatchers have noticed the badge exactly once, during the [[incident-2026-03-white-screen-safari]] rollback window, which was unrelated.

## Protocol details worth knowing before changing anything

The first frame after the ticket handshake is a `hello` from the server:

```json
{ "v": 1, "kind": "hello", "server_time": "2026-05-12T09:14:20.001Z", "org": "8a4f2c17-...", "last_seq": 8812, "replay_window_s": 600 }
```

`server_time` feeds `ServerClock` on the front (the same idea as the driver app's corrected clock), so relative times on the board do not drift when a dispatcher's laptop clock is wrong, which happens more than expected on shared depot machines. `last_seq` lets the client decide immediately whether it needs a resume, and `replay_window_s` is advertised rather than hard-coded so it can be tuned server-side without a front release.

The client answers with `{ "v": 1, "resume": <seq> }` or `{ "v": 1, "resume": null }` on first connect. The server then sends either the replayed events in order, or `{ "kind": "resync" }`, then a `{ "kind": "ready" }` marker. Until `ready`, the client buffers invalidations and applies them in one batch, otherwise a replay of 400 events produced 400 refetch triggers before the debounce kicked in.

`v` is 1 and there is no negotiation. The planned v2 (HF-1580) batches events into arrays and adds a `carrier.position` compaction (only the latest position per carrier in a replay), which would shrink replays by 80 % according to the recorded traffic of one week. It is not scheduled because the current replays are small enough. When v2 happens, the server will honour the `v` the client sends for one release cycle, then drop v1.

Ping and pong are text frames `{ "kind": "ping" }` and `{ "kind": "pong", "t": <ms> }` rather than WebSocket control frames, because the browser API does not expose control frames and we wanted the round-trip time measured in application code. The measured round trip from the office is 8 ms, from a depot on a shared connection 60 to 200 ms, and the number is shown in the connection badge's tooltip, which support uses to tell a slow network from a slow server.

The ticket endpoint returns 429 above 10 tickets per minute per user. A client in a reconnect loop (seen once with a broken proxy that closed every socket after 5 s) therefore stops hammering after a minute and shows the "live updates unavailable" badge instead of a spinner.
