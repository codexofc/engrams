# Contributing to Kept

Thanks for considering a contribution. This document is short on purpose: the rules
below are the ones that keep the project honest, and there are few of them.

## Ground rules

1. **Files are the truth.** Notes are plain markdown owned by the user. Nothing the
   engine writes under `.kept/` is precious, and no change may make the notes
   depend on the engine.
2. **No silent errors.** A plausible wrong vector is worse than a crash. Refuse an
   unknown pooling, a mismatched index, a non-finite vector. If you add a fallback,
   it must announce itself.
3. **Measure before you claim.** A change to ranking, chunking, tokenisation or the
   model path comes with a benchmark line in `docs/BENCHMARKS.md`, including the
   variants you tried and discarded. A number without its error bar is not a result.
4. **Zero personal data in the repository.** Fixtures are synthetic or public. No
   corpus excerpts, no hostnames, no tickets, no tokens. `kept secrets` must pass
   on anything you add.
5. **English everywhere.** Identifiers, comments, messages, documentation.

## Development

```sh
git clone https://github.com/codexofc/kept
cd kept
cargo build --release
cargo test            # model-dependent tests skip themselves when the model is absent
kept init ~/tmp-notes   # optional: downloads the model, enables the full suite
```

The full suite (concordance with the reference vectors, tokenizer parity on a corpus,
parallel determinism) needs the model in `~/.kept/models/` or `KEPT_MODEL`.

Before opening a pull request:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

CI runs exactly these on Linux and macOS, measures line coverage with `cargo llvm-cov`,
and writes the badge of the README to the `badges` branch at every push to `master`.

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

`examples/bench.rs` reads a JSON file of queries (`KEPT_BENCH`, default
`<root>/.kept/bench-queries.json`) and reports each family separately. Two public corpora
live under `bench/corpora/` (a large one, 523 notes, and a small one, 30 notes, both
synthetic, bilingual, with their blind queries): `scripts/bench-corpus.sh large` replays
the benchmark and the context cost measurement on a copy. A change to ranking must
be measured on them, and on your own notes if you have some. Write
queries blind, seeing only the target passage (`examples/draw_targets.rs` produces
the skeleton). Keep families apart and never merge them into one score. Ablations are
environment variables so they run on the same index. See `docs/BENCHMARKS.md`.

## Commits and pull requests

- One change per pull request, with the reasoning in the description.
- Subject line in the conventional form (`feat(index): …`, `fix(tokenizer): …`).
- Link the benchmark line or the test that proves the change.
- Keep the history readable: rebase on `develop`, no merge commits in a branch.

## Branches and releases

- `master` holds released code only. Every commit on it is a release or a hotfix,
  and every release is a tag `vX.Y.Z` on it.
- `develop` is where pull requests land. Open a branch from it (`feat/…`, `fix/…`,
  `docs/…`) and target it in the pull request. CI runs on both branches.
- A release is a pull request from `develop` to `master` that bumps the version in
  `Cargo.toml` and moves the `Unreleased` section of `CHANGELOG.md` under the new
  version. Once merged, the tag
  triggers the release workflow: binaries for four targets, release notes taken from
  the changelog, and the container image on GitHub Packages.
- The rulesets in `.github/rulesets/` protect both branches and the tags (pull
  request and green CI before merging into `master`, no force push, no deletion).
  Import them from the repository settings, or run `scripts/apply-rulesets.sh`.

## Reporting a bug

Open an issue with the command, the output, `kept version`, `kept status`, and
the platform. If the bug involves a ranking, attach the query and the expected note
(anonymised). A reproducible case is worth more than a description.

## Licence

By contributing you agree that your contribution is licensed under the same terms
as the project, MIT or Apache-2.0 at the user's choice.
