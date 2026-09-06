#!/bin/sh
# Creates or updates the repository rulesets from .github/rulesets/*.json.
# Needs the gh CLI logged in with admin rights. Rulesets are refused on a private
# repository without GitHub Pro: make the repository public first, or import the
# files from Settings > Rules > Rulesets.
set -eu
repo="${1:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
existing="$(gh api "repos/$repo/rulesets" --jq '.[] | "\(.name)\t\(.id)"')"
for file in "$(dirname "$0")"/../.github/rulesets/*.json; do
  name="$(sed -n 's/^ *"name": *"\([^"]*\)".*/\1/p' "$file" | head -1)"
  id="$(printf '%s\n' "$existing" | awk -F'\t' -v n="$name" '$1 == n { print $2 }')"
  if [ -n "$id" ]; then
    gh api -X PUT "repos/$repo/rulesets/$id" --input "$file" >/dev/null && echo "updated $name"
  else
    gh api -X POST "repos/$repo/rulesets" --input "$file" >/dev/null && echo "created $name"
  fi
done
