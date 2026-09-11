# Server mode

The specification of the second storage mode announced in [ROADMAP.md](ROADMAP.md):
one memory for a whole engineering team, on a database the company hosts, behind
the same CLI, MCP tools and hook as the file mode. Every figure here comes from a
measurement on the current engine or from a source named in
[BENCHMARKS.md](BENCHMARKS.md), and every choice says what it cost to decide.

## Why a second mode, in numbers

The file mode is a single machine, a single writer, and an index that is scanned
exhaustively at every search. Measured on my own memory (324 notes, 6 658 vectors
of 768 dimensions when the indexed questions are counted):

| step of one search | time | share |
|---|---|---|
| encoding the query with the 278 M model, on the laptop | about 180 ms | 97 % |
| exhaustive cosine scan over 6 658 vectors, no SIMD | 5.2 ms | 3 % |
| ranking, bonuses, output | under 1 ms | |

Two conclusions drive the design. First, an approximate index buys nothing under
100 000 vectors: the scan is not the cost. Extrapolated linearly, the scan reaches
55 ms at 70 000 vectors (ten developers, two years), 160 ms at 200 000 and 780 ms
at one million. An approximate index becomes necessary somewhere between the second
and the third line, and Postgres brings one. Second, the query encoding is the cost,
and it is paid on every developer's laptop on every prompt. Moving it to one server
with a GPU brings it to about ten milliseconds, an order of magnitude I have not
measured yet, and removes the 220 MB model and the 200 MB warm process from every
workstation. The hook budget goes from about 185 ms to about 20 to 30 ms on a local
network. That is the main performance argument for the server mode, whatever the
database.

What the file mode cannot do and a team needs: several authors writing at once,
notes with an owner and a scope (personal, project, team, company), a lexical
channel next to the vectors, learning tables per scope so that one developer's
confirmations do not move another's results, audit of who wrote and read what, and
backups that are not a git repository on one laptop.

## Choice of the database

I compared seven candidates on the criteria a company weighs. The number of crates
is measured in an empty Rust project with the version current in September 2026.
Kept pulls 60 today.

| candidate | mode | crates | concurrent authors | scopes and rights | index at 200 k | lexical | offline on the laptop | maturity |
|---|---|---|---|---|---|---|---|---|
| SQLite, rusqlite bundled | file | 10 | no | no | no | FTS5 compiled in | yes | complete |
| SQLite + sqlite-vec | file | 12 | no | no | brute force only | FTS5 | yes | crate 0.1.10-alpha, pre-v1 |
| Turso, Rust rewrite of SQLite | file + push/pull sync | 310 | yes, logical CDC | not native | on the roadmap | experimental | yes, by design | 0.8.0-pre, "keep independent backups until 1.0" |
| libSQL | file + embedded replica | similar | yes | no | DiskANN | no | yes | maintained, "new features go to Turso" |
| LanceDB | directory or object store | 571 | optimistic, bounded retries | no | IVF-PQ | tantivy | yes | 0.38, Arrow + DataFusion |
| Qdrant Edge + Qdrant server | in-process + server | 408 | via the server | API keys, limited JWT | filterable HNSW, int8 | sparse vectors | partly | Edge in beta, API moving |
| PostgreSQL + pgvector | server | 71 client side | transactions | row-level security | HNSW, halfvec | tsvector, or pg_search for BM25 | no | complete |

Rejected after reading: DuckDB vss (HNSW persistence experimental, no WAL recovery,
index held in RAM outside the memory limit), SurrealDB (open issue in v3.2.0, resident
memory at nine times the dataset), Chroma and Milvus Lite (Python).

**Decision: PostgreSQL** for the server mode. It is the only candidate that carries
concurrency, rights, the lexical channel and operations in one system, and a team
that runs it already has the skills, the backups and the monitoring. At 200 000
vectors of 768 dimensions the HNSW index sits in 1 to 2 GB of server memory, less in
`halfvec`, which is ordinary. Its known weakness, filters applied after the graph
walk, is handled below by partitioning the index per scope where selectivity is
high. The file mode stays the solo mode of the open-source product, moved from the
flat index to SQLite in the first lot so that both modes sit behind one trait. Turso
is the card to play again when it reaches 1.0, for the case where a developer must
work offline: it is the only candidate whose model is a local file that syncs.

## Architecture

```
developer's machine                          company network
┌──────────────────────────┐                 ┌──────────────────────────────┐
│ kept CLI · MCP · hook    │  HTTPS + token  │ kept server (Rust)           │
│ no model, no index       │ ──────────────▶ │  · encodes queries and notes │
│ cached hot index/project │ ◀────────────── │  · retrieval pipeline        │
│ ~/.kept/env: KEPT_SERVER │                 │  · scopes, audit, learning   │
└──────────────────────────┘                 │         │                    │
                                             │         ▼                    │
                                             │ PostgreSQL + pgvector        │
                                             │ notes · chunks · vectors     │
                                             │ tsvector · feedback · audit  │
                                             └──────────────────────────────┘
```

- **The client stays the same binary.** `kept search`, `answer`, `read`, `write`,
  `append`, `verify`, `supersede`, `link`, `learn`, `hook`, `mcp` keep their names,
  arguments and output. Only the store behind them changes, selected by
  `KEPT_SERVER` in `~/.kept/env`. Without it, the file mode runs as today.
- **The server is one Rust process**, the same crate built with a `server` feature,
  serving HTTPS with a per-developer token. It holds the embedding model (GPU when
  available, CPU otherwise) and runs the retrieval pipeline. Developers never get
  database credentials.
- **The database is PostgreSQL 16 or later with pgvector 0.8 or later.** Nothing
  else: no queue, no cache server, no second search engine.
- **The hot index of a project** (the `MEMORY.md` the agents auto-load) is rendered
  by the server and cached on the client, refreshed when the project's notes
  change. The hook keeps working when the server is unreachable, from the cache,
  and says so in one line.

## The storage trait

Both modes implement one trait, so that the benchmark, the CLI and the MCP tools
are written once.

```rust
pub trait Store {
    fn put_note(&mut self, scope: &Scope, note: &Note, chunks: &[Chunk]) -> Result<NoteId>;
    fn get_note(&self, scope: &Scope, name: &str) -> Result<Option<Note>>;
    fn supersede(&mut self, scope: &Scope, old: &str, new: NoteId) -> Result<()>;
    fn link(&mut self, a: NoteId, b: NoteId) -> Result<()>;
    fn search(&self, q: &Query, scope: &Scope, k: usize) -> Result<Vec<NoteHit>>;
    fn record_feedback(&mut self, scope: &Scope, query: &str, note: NoteId, source: Source) -> Result<()>;
    fn log(&mut self, event: &Event) -> Result<()>;
    fn hot_index(&self, project: &str) -> Result<String>;
}
```

`Query` carries the text, its vector, the identifiers found in it and the caller.
`FileStore` is the current layout, then SQLite. `PgStore` is the server mode. The
learned bonus, the identifier bonus and the window rule stay in the engine, above
the trait, so that both modes rank the same way.

## Schema

```sql
create table scopes   (id serial primary key, kind text not null check (kind in ('personal','project','team','company')),
                       name text not null, owner text, unique (kind, name));
create table members  (scope_id int references scopes, login text not null, role text not null check (role in ('reader','writer','curator')),
                       primary key (scope_id, login));
create table models   (id serial primary key, name text not null, weights_hash text not null, dim int not null, unique (name, weights_hash));
create table notes    (id bigserial primary key, scope_id int not null references scopes, name text not null,
                       family text not null, project text not null, type text not null, status text not null default 'active',
                       description text not null, body text not null, author text not null,
                       verified_at date, superseded_by bigint references notes, source text,
                       created_at timestamptz not null default now(), updated_at timestamptz not null default now(),
                       unique (scope_id, name));
create table links    (a bigint references notes, b bigint references notes, primary key (a, b));
create table chunks   (id bigserial primary key, note_id bigint not null references notes on delete cascade,
                       ordinal int not null, kind text not null check (kind in ('paragraph','question')),
                       text text not null, hash bigint not null,
                       tsv tsvector generated always as (to_tsvector('simple', text)) stored,
                       model_id int not null references models, embedding halfvec(768) not null,
                       unique (note_id, ordinal, kind, hash));
create table feedback (scope_id int references scopes, query text not null, note_id bigint references notes,
                       source text not null, count int not null default 1, last_day date not null,
                       primary key (scope_id, query, note_id));
create table events   (id bigserial primary key, at timestamptz not null default now(), login text not null,
                       kind text not null, query text, note_id bigint, ms int);

create index chunks_embedding on chunks using hnsw (embedding halfvec_cosine_ops) with (m = 16, ef_construction = 128);
create index chunks_tsv on chunks using gin (tsv);
create index chunks_note on chunks (note_id);
```

Choices in that schema:

- **Notes stay whole and readable.** The body is the markdown of today without its
  header, the header fields are columns. `kept export` writes the files back in the
  current layout, so nothing is locked in.
- **Questions are chunks** of kind `question`, attached to the paragraph they answer,
  exactly as the file mode indexes them next to it (`path#ordinal?k`).
- **`halfvec(768)`** halves the index and the memory for a recall the bench must
  confirm: the Q8 quantisation of the model itself cost nothing measurable, half
  precision on the stored vectors should not either, and the first lot measures it.
- **One `models` row per weights hash.** A model change re-embeds in the background,
  new rows carry the new model, search filters on the current model, and the old rows
  are dropped when the re-embedding completes. That is the file mode's "any mismatch
  rebuilds" without the downtime.
- **Row-level security** on `notes`, `chunks` and `feedback` keyed on `members`: a
  personal scope is visible to its owner, a project scope to its members, the company
  scope to everyone. The server connects as one role and sets the caller's login per
  transaction. Where a scope is small and selective, the partial HNSW index per scope
  avoids pgvector's post-filtering loss.
- **`unique (scope_id, name)`** replaces the file name. The near-duplicate refusal of
  `kept write` runs against the whole scope the author can see, not against one
  author's files.

## Retrieval pipeline

The server runs, in order, and the numbers are the budget on a local network:

| step | what | budget |
|---|---|---|
| encode | query vector on the server, GPU | about 10 ms, to be measured |
| vector channel | top 50 chunks by cosine, HNSW, scope filter | 2 to 5 ms |
| lexical channel | top 50 chunks by `ts_rank_cd` on `tsv`, same filter | 2 to 5 ms |
| fusion | reciprocal rank fusion of the two lists, k = 60 | under 1 ms |
| note ranking | max chunk score per note, identifier bonus, learned bonus of the caller's scope inside the 0.10 window | under 1 ms |
| rerank, optional | cross-encoder on the top 20, only if it keeps the isolated call under 0.3 s | 50 to 150 ms |
| answer | top 5 notes, passages, hot index refresh if stale | under 1 ms |

The lexical channel is the first measured gain of the server mode: on the public
corpora the words-only baseline already beats the vectors on some identifier queries.
The fusion is measured on the four families before it ships, against vectors alone,
and it stays out if it does not win.

## Learning and curation, per scope

- The learned table becomes `feedback` keyed by scope. A confirmation inside a
  project moves that project's ranking, a confirmation in the company scope moves
  everyone's. The two signals stay what they are today: an explicit `kept learn`, or
  a read that follows a search which had missed the note. Nothing is learned from the
  engine's own results.
- Curation becomes a server job with the same output as `bin/curation` today: notes
  whose verification expired, contradictions between two scopes on the same subject
  (two active notes with a cosine above 0.90 and different authors), notes never read
  in a year, and it writes a list, not a change. A curator role acts on it.
- `kept why`, `kept since` and `kept check` read the `events` and `notes` tables and
  keep their output.

## Migration and import

- `kept migrate --to server`: pushes every note of the file mode into the caller's
  personal scope, keeps the files as an export, and switches `~/.kept/env`. Reversible
  with `kept export`.
- `kept import` from Claude Code, Gemini, Codex and plain folders, as specified in
  the roadmap, is the first day of adoption for every developer and must exist
  before the server mode is offered to a team.
- Moving a note from a personal scope to a project scope is `kept share <note>
  <scope>`, a copy with provenance, the personal one superseded by it.

## Operations

- One PostgreSQL, one `kept server`, both self-hosted. No cloud service, no
  telemetry, no account, as the roadmap states.
- Sizing for 30 developers over two years: about 10 000 notes, 200 000 chunks with
  their questions, 300 MB of `halfvec` rows, 1 to 2 GB of HNSW in memory, a server
  process under 1 GB with the model loaded on CPU. A GPU is optional and changes the
  hook latency, not the recall.
- Backups are `pg_dump` plus the model weights, and `kept export` on a schedule if
  the team wants a readable copy in git.
- The server exposes a `/status` line for the tray and for `kept status --short`.

## Benchmark in server mode

The same `examples/bench` and `examples/tokens` run against a `PgStore`, on three
corpora: the two public ones, and the company's own memory anonymised by the team
that owns it, since the public corpora are written in the format and a team's memory
is not. Families stay separate. The report adds what the file mode does not have:
p50 and p99 of the hook end to end from a developer's machine, with and without the
GPU, and the recall of `halfvec` against `vector`.

## Delivery, in lots, each with its measurement

1. **Storage trait and SQLite file store.** Same bench figures as today on the three
   corpora, index rebuild time, FTS5 available but unused yet.
2. **Postgres store and the server**, personal scope only, token auth, encoding on the
   server. Measure: hook p50 and p99 from a laptop, recall of `halfvec`.
3. **Lexical channel and fusion.** Measure: the four families with and without it.
4. **Scopes, rights, learning per scope, `kept share`.** Measure: nothing lost on
   recall, the RLS cost on the search budget.
5. **`kept migrate` and `kept import`.** Measure: the degraded public corpus of the
   roadmap, before and after conversion.
6. **Curation job and audit views.** Measure: the curation list of a real team against
   what its curator would have listed.
7. **Cross-encoder rerank**, only if lot 3 leaves a gap on the detail family and a
   model fits the 0.3 s budget.

## Not decided here

- The model the server runs. The default stays the multilingual 278 M model. The
  company model trained on the team's own pairs is a separate experiment, planned in
  [MODEL-TUNING.md](MODEL-TUNING.md), and the server picks it up through the model
  registry without a code change.
- Offline work on a laptop in server mode. The cached hot index covers the hook, not
  writes. Turso at 1.0 is the candidate for a local file that syncs, and the trait is
  written so that a third store can come without touching the agents.
