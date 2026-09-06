# Changelog

All notable changes to Engrams are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed
- The context cost is measured at the three moments an agent touches the memory
  (the hook at every prompt, a consultation, a session start), against a keyword
  grep and an every-word grep, and aligned on the list prices of Anthropic, OpenAI
  and Google as published on 2026-09-06, in the README and in docs/BENCHMARKS.md.

## [0.4.0] - 2026-09-06

### Added
- The tray on Linux, through the StatusNotifierItem protocol over D-Bus in pure Rust
  (ksni): no GTK at build or run time, every release binary ships the `tray` feature.
- `examples/tokens.rs` measures what reaches the model's context to answer the
  benchmark queries, grep against the engine, and the size of the hot index against
  its unbounded equivalent. On the reference corpus: 33 600 tokens per question with
  grep, reaching the note 35 times out of 96, against 2 350 with `engram search` and
  `engram read` (78 out of 96) and 520 with `engram answer` alone (70 out of 96).

## [0.3.1] - 2026-09-06

### Fixed
- The ModernBERT model shipped since 0.2.0 under the name granite-embedding-small-english-r2
  was in fact `ibm-granite/granite-embedding-97m-multilingual-r2`: 97 M parameters,
  180 000-piece vocabulary, SiLU, 32 768-token window, multilingual. The registry,
  the alias (`granite-multilingual-r2`), the concordance fixture, the CI download and
  every document now name it correctly. The measurements were made on that model and
  stand. The real granite-embedding-small-english-r2 (47 M, English, GELU, mean
  pooling) is not in the registry yet, it has not been measured.
- The coverage job of the CI downloads the two models the tests need, so the badge
  counts the concordance, parity and end-to-end tests.

## [0.3.0] - 2026-09-06

### Added
- Model registry with seven checkpoints: `engram models` lists them, `engram models
  use <alias>` switches, the guided setup and `engram init --model <alias>` accept the
  same aliases. New choices: multilingual-e5-small, -base and -large, granite-embedding-english-r2,
  gte-modernbert-base. The e5 prefixes (`query: `, `passage: `) are applied by the
  engine and recorded in the index header, so a prefix change rebuilds the index.
- BERT encoders (MiniLM and friends) through the XLM-RoBERTa graph with absolute
  positions, and a Unigram loader that accepts a `Sequence` normaliser.
- `ENGRAM_IDLE=never` keeps the warm process resident. `engram config` offers the
  idle policy as a menu (5 minutes, 30 minutes, 2 hours, never), and `engram status
  --short` prints one line while the process runs, for a shell prompt or a status bar.
- `engram tray`: the Engrams mark in the menu bar (macOS) or the system tray, in
  colour while the warm process runs, with a menu to reindex, open the notes, start
  or stop the process. `engram tray install` starts it at login. Cargo feature `tray`,
  on in the macOS release binaries.
- Container image on GitHub Packages (`ghcr.io/codexofc/engrams`) built from the
  release binaries for linux/amd64 and linux/arm64.
- Coverage job in CI, `develop` branch and repository rulesets ready to import.
- Seven models measured on the same corpus, in quality per query family and in
  isolation (latency, peak memory): tables and two charts in docs/BENCHMARKS.md,
  drawn by `examples/charts.rs` from the typed-in numbers. Every table and chart
  names its model.

### Fixed
- The compact Unigram tokenizer honours the `Metaspace`-only layout and the
  space-folding normaliser of multilingual-e5-small (66 mismatches out of 2 073
  texts before). The parity test covers every installed tokenizer.
- Indexes written before the prompts field existed stay valid after the upgrade.

### Changed
- Minimum Rust version 1.98. Intel macOS binaries are cross-compiled from the Apple
  silicon runner.
- The project is written Engrams in prose, `engram` stays the command.

### Removed
- bge-m3 from the candidate list: the repository ships no safetensors weights.

## [0.2.0] - 2026-09-06

### Added
- ModernBERT encoders, with a graph written after the reference implementation
  (activation read from the configuration, lazily read embedding table, Q8 option).
  Cosine 1.000000 with the reference vectors in F32, 0.99986 in Q8. On the
  reference corpus, text only: 83 / 54 / 100 / 72 % against 83 / 58 / 75 / 81 %
  for the multilingual model, 0.43 s and 285 MB per isolated search.
- Byte-level BPE tokenizer, identical to the reference crate on a 2 000-text corpus.
- Model choice in the guided setup and `engram init --model <repo>`: multilingual
  512-token window (default) or English 8192-token window.
- Tool detection in the guided setup: Claude Code, Codex CLI, opencode, Gemini CLI,
  Cursor, Windsurf, Kandev, with `engram setup <tool|all>`.
- `engram config` to list, set, unset and edit the settings kept in `~/.engram/env`.
- Terminal logo rasterised from the README mark, coloured help, `install.sh`.
- Issue and pull request templates, this changelog.

### Changed
- The wizard no longer initialises git inside an existing repository.

## [0.1.0] - 2026-09-06

### Added
- First public release: local semantic memory over markdown notes, XLM-RoBERTa
  encoder on the CPU (198 MB resident, 0.10 s per search), chunking by paragraph,
  indexed questions, bounded learning from usage, hot `MEMORY.md` indexes, warm
  process, Claude Code hook, MCP server over stdio, life-cycle commands, secret
  scanner, curation report, benchmark tooling.

[Unreleased]: https://github.com/codexofc/engrams/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/codexofc/engrams/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/codexofc/engrams/releases/tag/v0.1.0
