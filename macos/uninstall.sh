#!/bin/bash
# Removes the user-level install. Remove the input source in System Settings if it lingers.
set -euo pipefail
killall SinhalaInput 2>/dev/null || true
rm -rf "$HOME/Library/Input Methods/SinhalaInput.app"
echo "Removed. Log out and back in to clear it from the input menu."
