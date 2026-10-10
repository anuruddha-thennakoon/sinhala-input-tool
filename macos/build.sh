#!/bin/bash
# Builds macos/build/SinhalaInput.app (universal arm64 + x86_64, ad-hoc signed).
#   HOST_ONLY=1 macos/build.sh   this machine's architecture only
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

APP="$OUT/SinhalaInput.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

# Universal binary: build each architecture separately, then merge with lipo.
# HOST_ONLY=1 builds just this machine's architecture (faster for development).
if [ "${HOST_ONLY:-0}" = "1" ]; then
    ARCHS=("$(uname -m)")
else
    ARCHS=(arm64 x86_64)
fi
SLICES=()
for arch in "${ARCHS[@]}"; do
    case "$arch" in
        arm64)  triple=aarch64-apple-darwin ;;
        x86_64) triple=x86_64-apple-darwin ;;
    esac
    echo "==> Rust core ($triple)"
    (cd "$ROOT" && cargo build --release -p sinhala-core --target "$triple")
    echo "==> Swift ($arch)"
    swiftc -O -swift-version 5 -target "$arch-apple-macos12.0" \
        -import-objc-header "$HERE/Sources/Bridging-Header.h" -Xcc -I"$ROOT/core/include" \
        "$HERE"/Sources/*.swift \
        "$ROOT/target/$triple/release/libsinhala_core.a" \
        -o "$OUT/SinhalaInput-$arch"
    SLICES+=("$OUT/SinhalaInput-$arch")
done
lipo -create "${SLICES[@]}" -output "$APP/Contents/MacOS/SinhalaInput"
rm -f "${SLICES[@]}"
echo "Architectures: $(lipo -archs "$APP/Contents/MacOS/SinhalaInput")"

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
