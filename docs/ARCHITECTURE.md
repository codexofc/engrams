# Architecture

## Guiding principle

The markdown files are the truth. Vectors, hot indexes and caches derive from them
and are disposable: losing `.kept/` costs a few minutes of rebuild, losing the notes
would cost years. Nothing the engine writes is precious, and nothing it writes may
make the notes depend on it.

Corollaries: the index refuses vectors produced under other conditions (model,
weights, dimension, pooling) instead of mixing them; freshness is read from the
frontmatter at search time, never from the index; a fact is replaced, never deleted.

## Layout on disk

```
<root>/                          the notes, KEPT_ROOT or ~/.kept/root
  <family>/<project>/<name>.md   one durable fact per file, frontmatter + body
  <family>/<project>/MEMORY.md   generated hot index, bounded to 17 KB
  <family>/common/               notes listed in every project of the family
  .kept/
    index.bin                    vectors, binary (JSON header + f32 LE)
    questions.json               paragraph fingerprint -> generated questions (versionable)
    feedback.json                learned (query, note) pairs (versionable)
    feedback.bin                 cached vectors of the learned queries
    usage.log, searches.log, reads.log
    kept.sock                  warm process socket
~/.kept/models/<model>/        weights, config, pooling, sentencepiece.bpe.model
```

## Modules

| module | responsibility | depends on |
|---|---|---|
| `note` | frontmatter and body of a note, YAML subset | nothing |
| `check` | naming rules, mandatory fields, links, YAML subset | `note` |
| `chunking` | split a body along markdown structure, context prefix | nothing |
| `tokenizer` | compact Unigram tokenizer, native SentencePiece reader, tokenizer choice | `spm_precompiled` |
| `bpe` | byte-level BPE tokenizer for ModernBERT checkpoints | `regex` |
| `xlm_roberta` | the XLM-RoBERTa graph, lazy embedding table, Q8 linear layers | `candle` |
| `modernbert` | the ModernBERT graph: rotary positions, global and sliding-window attention, gated MLP | `candle` |
| `model`, `pooling` | model configuration and pooling read from the model | `serde_json` |
| `embedder` | load a model directory, encode texts, parallel encoding | the above |
| `index` | flat vector storage, freshness by content hash, atomic save | `serde` |
| `similarity` | cosine, ranking per note by maximum, identifier bonus | nothing |
| `feedback` | bounded learning from explicit and missed signals | nothing |
| `questions` | generated questions cache and external command | nothing |
| `duplicates` | near-duplicate notes from the index alone | `index` |
| `hot` | the generated `MEMORY.md` with strata and compaction | `note` |
| `lifecycle` | append, verify, supersede, relink on the file text | `check` |
| `secrets` | patterns that must never be written | `regex` |
| `paths` | where things live | nothing |
| `models` | the registry of known checkpoints: aliases, families, sizes, licences, the prefixes some expect | nothing |
| `tray` (binary, feature `tray`) | the mark in the menu bar or system tray, its menu, the login item | `tray-icon`, `tao` on macOS, `ksni` on Linux |

The binary (`src/main.rs`) composes them: refresh pass, engine, warm process, CLI,
hook, MCP server. Pure modules have no I/O and are tested without a model.

## The refresh pass

The same pass serves `kept index`, every search, and the warm process's
background loop:

1. walk the notes, split each body into chunks, prefix each chunk with the note's
   name and description;
2. for every chunk key (`path#ordinal`) whose note fingerprint changed, queue the
   text; for every cached question of that chunk (`path#ordinal?k`), queue it too;
3. optionally ask the questions command for chunks without cached questions, in
   batches, saving the cache after each batch;
4. drop index entries that no longer correspond to a chunk (ghosts);
5. embed the queue on all cores, in input order, and store the vectors.

A search therefore never returns a stale vector: the model is loaded anyway, and
re-embedding the few notes that moved costs less than a wrong answer.

## Search

Encode the query, compute the cosine with every vector (normalised at indexing, so a
dot product), aggregate per note by maximum, add the bonuses (lexical on identifiers,
learned from usage within the window of the top score), rank, hide archived notes,
show five with the passage that matched.

## Warm process

`kept serve` keeps the engine loaded behind a Unix socket and exits after
`KEPT_IDLE` seconds without a request (`never` keeps it resident, `kept status
--short` shows it in one line). Protocol: one tab-separated request line,
one reply. Each connection is served in its own thread under a read lock; refreshes
take the write lock. A client whose binary is newer than the server's makes the
server stop, so `cargo install` takes effect at the next call. The first search
starts the process and answers locally without waiting for it.

## Integrations

- `kept hook` reads Claude Code's `UserPromptSubmit` JSON on stdin and prints a
  `<working-memory>` block with the passages close to the prompt.
- `kept mcp` is a JSON-RPC server over stdio (one message per line) exposing
  `search`, `answer`, `read`, `write`, `append`, `link`, `learn`. Tools call the same
  functions as the CLI.
- `kept setup <tool>` edits the tool's configuration to register both.

## Formats

**Index** (`index.bin`): magic `MEMIDX1\n`, a u32 LE length, a JSON object
`{header, entries}`, then `entries.len() * dim` f32 LE. Written to a temporary file
and renamed. The JSON form of the same structure is still readable.

**Questions cache** (`questions.json`): a sorted map from the fingerprint of the
prefixed chunk text to its questions. Sorted keys keep diffs small under version
control.

**Feedback table** (`feedback.json`): a list of `{query, path, source, count,
last_day}`. Readable and editable by hand.
