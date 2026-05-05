#!/usr/bin/env sh
# one-shot installer for drift. resolves the latest release, picks the right
# {os, arch} archive, drops the binary into ~/.local/bin (or /usr/local/bin if
# you have write access). usage:
#   curl -fsSL https://drift.frkhd.com/install.sh | sh
#   curl -fsSL https://drift.frkhd.com/install.sh | sh -s -- --bin-dir /opt/bin

set -eu

REPO="f4rkh4d/drift"
BIN_DIR=""
VERSION="latest"

while [ $# -gt 0 ]; do
    case "$1" in
        --bin-dir) BIN_DIR="$2"; shift 2 ;;
        --version) VERSION="$2"; shift 2 ;;
        -h|--help)
            cat <<EOF
usage: install.sh [--bin-dir DIR] [--version vX.Y.Z]
defaults: bin-dir = first writable of /usr/local/bin, ~/.local/bin
          version = latest released
EOF
            exit 0
            ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

uname_s=$(uname -s 2>/dev/null || echo unknown)
uname_m=$(uname -m 2>/dev/null || echo unknown)
case "$uname_s" in
    Linux)  os=linux ;;
    Darwin) os=macos ;;
    *) echo "drift install: unsupported os $uname_s" >&2; exit 1 ;;
esac
case "$uname_m" in
    x86_64|amd64)   arch=amd64 ;;
    aarch64|arm64)  arch=arm64 ;;
    *) echo "drift install: unsupported arch $uname_m" >&2; exit 1 ;;
esac

if [ "$VERSION" = "latest" ]; then
    VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
        | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)
    if [ -z "$VERSION" ]; then
        echo "drift install: could not resolve latest version" >&2
        exit 1
    fi
fi

asset="drift-${VERSION}-${os}-${arch}.tar.gz"
url="https://github.com/$REPO/releases/download/$VERSION/$asset"

if [ -z "$BIN_DIR" ]; then
    if [ -w /usr/local/bin ] 2>/dev/null; then
        BIN_DIR=/usr/local/bin
    else
        BIN_DIR="$HOME/.local/bin"
    fi
fi
mkdir -p "$BIN_DIR"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "drift: installing $VERSION ($os-$arch) into $BIN_DIR"
curl -fsSL "$url" -o "$tmp/$asset"
tar -xzf "$tmp/$asset" -C "$tmp"

if [ ! -x "$tmp/drift" ]; then
    echo "drift install: archive did not contain a drift binary" >&2
    exit 1
fi
mv "$tmp/drift" "$BIN_DIR/drift"
chmod +x "$BIN_DIR/drift"

echo "drift: $("$BIN_DIR/drift" --version 2>/dev/null || echo "$VERSION") installed at $BIN_DIR/drift"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "drift: note: $BIN_DIR is not on your PATH. add it to your shell rc." ;;
esac
