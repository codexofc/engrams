# Roadmap

What comes next, in the order I intend to do it. Nothing here is a promise with a
date. Each item ships with its measurement, as everything before it did.

## Two storage modes

Kept has one mode today, the **file mode**: markdown notes are the truth, the
index is derived and disposable, everything lives on one machine and in one git
repository. It is the right mode for a person and their agents, and it stays the
default.

The second mode, the **server mode**, is for a team: several people and several
agents writing to the same memory at once, corpora of tens of thousands of notes,
and a memory that outlives any one laptop. Its specification, with the database
comparison that decided it, the schema, the retrieval pipeline and the delivery
lots, is [SERVER-MODE.md](SERVER-MODE.md). In short:

- the same CLI, the same MCP tools and the same hook, unchanged, so that a project
  can move from one mode to the other without touching the agents,
- a store behind a trait, SQLite for the file mode and PostgreSQL with pgvector
  for the server mode, so that the choice is a configuration line,
- queries and notes encoded on the server, so that a developer's machine carries
  neither the model nor the index and the hook answers in tens of milliseconds,
- notes stay whole and readable: `kept export` writes them back as files, and
  nothing is locked in,
- scopes (personal, project, team, company) with rights, learning tables per
  scope, audit, and the same refusal of secrets and near-duplicates as the CLI,
- a lexical channel fused with the vector ranking, measured before it ships,
- the benchmark run in both modes on the same corpus, so that the database mode
  cannot silently lose recall or latency.

## Import an existing memory

Most people who try the engine already have a memory: a folder of markdown, the
memory directory of Claude Code, `GEMINI.md` or `AGENTS.md` files, notes without a
header and with several facts per file. The benchmarks assume the format
`kept check` enforces, so a raw import scores below them.

`kept import <path> --from claude-code|gemini|codex|plain --into <family/project>`:

- a raw mode that keeps every file whole, derives the name from the file name, the
  description from the first heading or line, the type by default, the date from
  the file, and adds the header without touching the body,
- a converting mode that splits a long file into one note per `##` section, skips
  index files (the `MEMORY.md` of Claude Code are pointers, not facts), reports
  near-duplicates of existing notes and refuses secrets,
- `--dry-run` first, always, with a report of what would be converted, split and
  left aside,
- a third public corpus in `bench/corpora/`, the large one degraded mechanically
  (headers removed, notes merged by project, descriptions lost), to measure what the
  engine gives on raw notes, what the import recovers, and the gap with the clean
  corpus.

## Measured on more machines

- A fresh-machine run of the installer and the guided setup on macOS and on a
  Linux desktop, with the tray displayed.
- Windows: the warm process uses a Unix socket, so Windows needs a named pipe or a
  local TCP socket before a binary can ship. WSL runs the Linux binary today.
- The integrations with Codex CLI, Gemini CLI, Cursor and Windsurf are written
  from their configuration formats and not yet exercised on the tools themselves.

## Retrieval

- A lexical channel (BM25 or a compact equivalent) merged with the vector ranking,
  measured on the four families: the words-only baseline already wins on some
  identifier queries.
- Cross-encoder reranking of the five results, if a small enough model exists to
  keep the isolated call under 0.3 s.
- Per-project learning tables, so that a confirmation in one project does not move
  another.

## Not planned

- No cloud service, no account, no telemetry. The database mode is something you
  host.
- No model training or fine-tuning inside the engine: the notes are the memory,
  and they stay readable by a person. Training a model on a team's own pairs is a
  procedure outside the engine, planned as an experiment in
  [MODEL-TUNING.md](MODEL-TUNING.md), and the engine only loads its weights.
