# Contributing to engrams

Thanks for considering a contribution. This document is short on purpose: the rules
below are the ones that keep the project honest, and there are few of them.

## Ground rules

1. **Files are the truth.** Notes are plain markdown owned by the user. Nothing the
   engine writes under `.engram/` is precious, and no change may make the notes
   depend on the engine.
2. **No silent errors.** A plausible wrong vector is worse than a crash. Refuse an
   unknown pooling, a mismatched index, a non-finite vector. If you add a fallback,
   it must announce itself.
3. **Measure before you claim.** A change to ranking, chunking, tokenisation or the
   model path comes with a benchmark line in `docs/BENCHMARKS.md`, including the
   variants you tried and discarded. A number without its error bar is not a result.
4. **Zero personal data in the repository.** Fixtures are synthetic or public. No
   corpus excerpts, no hostnames, no tickets, no tokens. `engram secrets` must pass
   on anything you add.
5. **English everywhere.** Identifiers, comments, messages, documentation.

## Development

```sh
git clone https://github.com/codexofc/engrams
cd engrams
cargo build --release
cargo test            # model-dependent tests skip themselves when the model is absent
engram init ~/tmp-notes   # optional: downloads the model, enables the full suite
```

The full suite (concordance with the reference vectors, tokenizer parity on a corpus,
parallel determinism) needs the model in `~/.engram/models/` or `ENGRAM_MODEL`.

Before opening a pull request:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

CI runs exactly these on Linux and macOS.

## Style

- `rustfmt.toml` sets the width; do not fight it.
- Doc comments say **why**, in one to three lines. What the code does is in the
  code. A comment that restates the next line is deleted in review.
- Prefer a plain function over a trait with one implementation, and the standard
  library over a dependency. Every new crate needs a sentence in the pull request
  explaining what it buys.
- Errors are `Result<_, String>` with a message a user can act on: name the file,
  the field, the command to run.
- Tests are named as sentences (`a_changed_file_is_not_fresh`) and one test checks
  one behaviour.

## Benchmarks

`examples/bench.rs` reads a JSON file of queries (`ENGRAM_BENCH`, default
`<root>/.engram/bench-queries.json`) and reports each family separately. Write
queries blind, seeing only the target passage (`examples/draw_targets.rs` produces
the skeleton). Keep families apart; never merge them into one score. Ablations are
environment variables so they run on the same index. See `docs/BENCHMARKS.md`.

## Commits and pull requests

- One change per pull request, with the reasoning in the description.
- Subject line in the conventional form (`feat(index): …`, `fix(tokenizer): …`).
- Link the benchmark line or the test that proves the change.
- Keep the history readable: rebase on `master`, no merge commits in a branch.

## Reporting a bug

Open an issue with the command, the output, `engram version`, `engram status`, and
the platform. If the bug involves a ranking, attach the query and the expected note
(anonymised); a reproducible case is worth more than a description.

## Licence

By contributing you agree that your contribution is licensed under the same terms
as the project, MIT or Apache-2.0 at the user's choice.
