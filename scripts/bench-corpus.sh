#!/bin/sh
# Replays the retrieval benchmark and the context cost measurement on one of the
# public corpora (bench/corpora/<name>), on a copy so that the repository stays
# clean. Needs the release build (`cargo build --release --examples`) and the
# default model in ~/.engram/models. Queries come from bench/corpora/<name>/bench-queries.json.
#
#   scripts/bench-corpus.sh large
#   scripts/bench-corpus.sh small
set -eu
name="${1:?corpus name: large or small}"
here="$(cd "$(dirname "$0")/.." && pwd)"
src="$here/bench/corpora/$name"
root="${TMPDIR:-/tmp}/engrams-bench-$name"
rm -rf "$root" && mkdir -p "$root"
cp -R "$src"/. "$root"/
mkdir -p "$root/.engram"
cp "$src/bench-queries.json" "$root/.engram/bench-queries.json"
export ENGRAM_ROOT="$root" ENGRAM_NO_DAEMON=1 ENGRAM_BIN="$here/target/release/engram"
unset ENGRAM_QUESTIONS_CMD
echo "# corpus $name: $(find "$src" -name '*.md' | wc -l | tr -d ' ') notes, $(du -sk "$src" | cut -f1) KB"
"$ENGRAM_BIN" index | tail -1
echo
"$here/target/release/examples/bench" | grep -v missed
echo
ENGRAM_ID_BONUS=0 "$here/target/release/examples/bench" | grep -v missed | sed 's/^# model/# text only, model/'
echo
ENGRAM_LEXICAL=1 "$here/target/release/examples/bench" | grep -v missed | sed 's/^# model/# words only, model/'
echo
"$here/target/release/examples/tokens"
