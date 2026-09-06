#!/bin/sh
# Installs the latest kept release for this platform, then starts the guided setup.
#   curl -sSfL https://raw.githubusercontent.com/codexofc/kept/master/install.sh | sh
# Set KEPT_INSTALL_DIR to choose the directory (default: ~/.local/bin).
set -eu

repo="codexofc/kept"
dir="${KEPT_INSTALL_DIR:-$HOME/.local/bin}"

os=$(uname -s)
arch=$(uname -m)
case "$os/$arch" in
  Linux/x86_64)   target="x86_64-unknown-linux-gnu" ;;
  Linux/aarch64)  target="aarch64-unknown-linux-gnu" ;;
  Linux/arm64)    target="aarch64-unknown-linux-gnu" ;;
  Darwin/arm64)   target="aarch64-apple-darwin" ;;
  Darwin/x86_64)  target="x86_64-apple-darwin" ;;
  *) echo "no prebuilt binary for $os/$arch; build with: cargo install kept" >&2; exit 1 ;;
esac

tag=$(curl -sSfL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)
[ -n "$tag" ] || { echo "could not read the latest release of $repo" >&2; exit 1; }
url="https://github.com/$repo/releases/download/$tag/kept-$tag-$target.tar.gz"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
echo "downloading kept $tag for $target"
curl -sSfL "$url" -o "$tmp/kept.tar.gz"
tar -xzf "$tmp/kept.tar.gz" -C "$tmp"
mkdir -p "$dir"
install -m 755 "$tmp/kept" "$dir/kept"
echo "installed $dir/kept"

case ":$PATH:" in
  *":$dir:"*) ;;
  *) echo "add $dir to your PATH, for example: export PATH=\"$dir:\$PATH\"" ;;
esac

if [ -t 0 ] && [ -t 1 ]; then
  exec "$dir/kept" init
else
  echo "next: run \`kept init\` for the guided setup"
fi
