#!/bin/bash
# Builds macos/build/SinhalaInput.app (ad-hoc signed, host architecture).
#   macos/build.sh         build the app
#   macos/build.sh test    run the KeyRouter unit tests only
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
OUT="$HERE/build"
mkdir -p "$OUT"

if [ "${1:-}" = "test" ]; then
    swiftc -swift-version 5 "$HERE/Sources/KeyRouter.swift" "$HERE/Tests/RouterTests/main.swift" -o "$OUT/router_tests"
    "$OUT/router_tests"
    exit 0
fi

echo "==> Rust core"
(cd "$ROOT" && cargo build --release -p sinhala-core)

APP="$OUT/SinhalaInput.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

echo "==> Swift"
swiftc -O -swift-version 5 \
    -import-objc-header "$HERE/Sources/Bridging-Header.h" -Xcc -I"$ROOT/core/include" \
    "$HERE"/Sources/*.swift \
    "$ROOT/target/release/libsinhala_core.a" \
    -o "$APP/Contents/MacOS/SinhalaInput"

cp "$HERE/Info.plist" "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist" >/dev/null

echo "==> Icon"
if ! swift "$HERE/tools/make_icon.swift" "$APP/Contents/Resources/icon.tiff" 2>/dev/null; then
    echo "   (icon generation failed; continuing without it)"
fi

echo "==> Sign (ad-hoc)"
codesign --force --sign - --deep "$APP"
codesign --verify --deep --strict "$APP"
echo "Built $APP"
