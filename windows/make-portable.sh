#!/bin/bash
# Cross-builds the Windows DLLs on macOS/Linux and packs windows/dist/SinhalaInput-<version>-portable.zip,
# ready to copy to a Windows PC (unzip, double-click Install.cmd).
#
# One-time setup:   cargo install cargo-xwin --locked
# (cargo-xwin downloads Microsoft's Windows SDK and C runtime on first use, ~1 GB; that download
#  is under Microsoft's licence terms.) Nothing here has been run on Windows by this script:
#  the CI run additionally registers, activates and uninstalls the DLLs on a real runner.
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$HERE")"
cd "$ROOT"

command -v cargo-xwin >/dev/null || { echo "Install cargo-xwin first: cargo install cargo-xwin --locked"; exit 1; }
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc >/dev/null

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' core/Cargo.toml | head -1)"
export XWIN_ARCH="x86,x86_64"
for target in x86_64-pc-windows-msvc i686-pc-windows-msvc; do
    echo "==> $target"
    cargo xwin build --release -p sinhala-tsf --target "$target" 2>&1 | grep -vE "LNK4099|failed to load reference|^\s*$|linker_messages|generated 1 warning|^\s*\|?\s*=" || true
    [ -f "target/$target/release/sinhala_tsf.dll" ] || { echo "build failed for $target"; exit 1; }
done

NAME="SinhalaInput-$VERSION-portable"
STAGE="$HERE/dist/$NAME"
rm -rf "$HERE/dist"
mkdir -p "$STAGE"
cp target/x86_64-pc-windows-msvc/release/sinhala_tsf.dll "$STAGE/sinhala_tsf_x64.dll"
cp target/i686-pc-windows-msvc/release/sinhala_tsf.dll "$STAGE/sinhala_tsf_x86.dll"
cp "$HERE/installer/sinhala.ico" "$STAGE/"
cp "$HERE/install.ps1" "$HERE/uninstall.ps1" "$HERE"/portable/* "$STAGE/"

# Windows batch files and text want CRLF line endings.
for f in "$STAGE"/*.cmd "$STAGE"/*.txt "$STAGE"/*.ps1; do
    sed -i '' 's/\r$//; s/$/\r/' "$f"
done

(cd "$HERE/dist" && zip -qr "$NAME.zip" "$NAME")
echo
ls -l "$HERE/dist/$NAME.zip"
unzip -l "$HERE/dist/$NAME.zip"
