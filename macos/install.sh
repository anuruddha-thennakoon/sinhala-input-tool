#!/bin/bash
# Installs the built app for the current user and enables the input source.
# After this, add "Sinhala" in System Settings > Keyboard > Input Sources if it is not already on.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
APP="$HERE/build/SinhalaInput.app"
DEST="$HOME/Library/Input Methods"
[ -d "$APP" ] || { echo "Run macos/build.sh first"; exit 1; }
mkdir -p "$DEST"
killall SinhalaInput 2>/dev/null || true
rm -rf "$DEST/SinhalaInput.app"
cp -R "$APP" "$DEST/"
swift "$HERE/tools/register.swift" "$DEST/SinhalaInput.app"
echo "Installed to $DEST. If it does not appear in the input menu, log out and back in."
