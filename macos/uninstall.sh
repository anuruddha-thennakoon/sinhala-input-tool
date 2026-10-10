#!/bin/bash
# Removes the input method (user-level install and, if present, the system-wide .pkg install).
set -euo pipefail
killall SinhalaInput 2>/dev/null || true
rm -rf "$HOME/Library/Input Methods/SinhalaInput.app"
if [ -d "/Library/Input Methods/SinhalaInput.app" ]; then
    sudo rm -rf "/Library/Input Methods/SinhalaInput.app"
    sudo pkgutil --forget com.itrowin.inputmethod.Sinhala.pkg 2>/dev/null || true
fi
echo "Removed. Log out and back in to clear it from the input menu."
echo "Your settings are kept in ~/Library/Application Support/SinhalaInput (delete it to reset)."
