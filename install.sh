#!/bin/sh
# Installs the latest engram release for this platform, then starts the guided setup.
#   curl -sSfL https://raw.githubusercontent.com/codexofc/engrams/master/install.sh | sh
# Set ENGRAM_INSTALL_DIR to choose the directory (default: ~/.local/bin).
set -eu

repo="codexofc/engrams"
dir="${ENGRAM_INSTALL_DIR:-$HOME/.local/bin}"

os=$(uname -s)
arch=$(uname -m)
case "$os/$arch" in
  Linux/x86_64)   target="x86_64-unknown-linux-gnu" ;;
  Linux/aarch64)  target="aarch64-unknown-linux-gnu" ;;
  Linux/arm64)    target="aarch64-unknown-linux-gnu" ;;
  Darwin/arm64)   target="aarch64-apple-darwin" ;;
  Darwin/x86_64)  target="x86_64-apple-darwin" ;;
  *) echo "no prebuilt binary for $os/$arch; build with: cargo install engrams" >&2; exit 1 ;;
esac

tag=$(curl -sSfL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)
[ -n "$tag" ] || { echo "could not read the latest release of $repo" >&2; exit 1; }
url="https://github.com/$repo/releases/download/$tag/engram-$tag-$target.tar.gz"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
echo "downloading engram $tag for $target"
curl -sSfL "$url" -o "$tmp/engram.tar.gz"
tar -xzf "$tmp/engram.tar.gz" -C "$tmp"
mkdir -p "$dir"
install -m 755 "$tmp/engram" "$dir/engram"
echo "installed $dir/engram"

case ":$PATH:" in
  *":$dir:"*) ;;
  *) echo "add $dir to your PATH, for example: export PATH=\"$dir:\$PATH\"" ;;
esac

if [ -t 0 ] && [ -t 1 ]; then
  exec "$dir/engram" init
else
  echo "next: run \`engram init\` for the guided setup"
fi
