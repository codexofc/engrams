# Changelog

All notable changes to engrams are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

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
