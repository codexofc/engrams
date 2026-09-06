## What this changes

<!-- One change per pull request. Say what and why, in a few sentences. -->

## How it was verified

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets` (with the model present, if the change touches encoding)
- [ ] Benchmark line added to `docs/BENCHMARKS.md` (required for ranking, chunking, tokenizer or model changes)

## Checklist

- [ ] No personal data, corpus excerpt or secret in the diff
- [ ] Doc comments say why, not what; English everywhere
- [ ] `CHANGELOG.md` updated under *Unreleased*
