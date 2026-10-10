#!/bin/bash
# Builds macos/build/SinhalaInput-<version>.pkg, which installs the input method system-wide
# into /Library/Input Methods (asks for an administrator password). Unsigned.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
APP="$HERE/build/SinhalaInput.app"
[ -d "$APP" ] || "$HERE/build.sh"
VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$APP/Contents/Info.plist")"

STAGE="$HERE/build/pkg-root"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"

PKG="$HERE/build/SinhalaInput-$VERSION.pkg"
pkgbuild --root "$STAGE" \
    --identifier com.itrowin.inputmethod.Sinhala.pkg \
    --version "$VERSION" \
    --install-location "/Library/Input Methods" \
    --scripts "$HERE/pkg-scripts" \
    "$PKG"
echo "Built $PKG"
