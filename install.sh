#!/bin/sh
# Installs the latest ziv release into /Applications (specs/update.md rule 3).
# curl sets no quarantine attribute, so Gatekeeper does not ask despite the
# ad-hoc signature. One-liner:
#   curl -fsSL https://raw.githubusercontent.com/davidbonan/ziv/main/install.sh | sh
set -eu

api="https://api.github.com/repos/davidbonan/ziv/releases/latest"
url="$(curl -fsSL "$api" | grep -o '"browser_download_url": *"[^"]*ziv-macos\.zip"' | head -1 | sed -E 's/.*"(https[^"]*)"/\1/')"
[ -n "$url" ] || { echo "error: no ziv-macos.zip asset in the latest release" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "Downloading $url"
curl -fSL "$url" -o "$tmp/ziv-macos.zip"
rm -rf /Applications/ziv.app
ditto -x -k "$tmp/ziv-macos.zip" /Applications
echo "Installed /Applications/ziv.app"
open /Applications/ziv.app
