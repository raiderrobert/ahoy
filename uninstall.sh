#!/bin/bash
set -e

# Ahoy uninstaller
# Usage: curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/uninstall.sh | bash

AHOY_HOME="$HOME/.ahoy"
AHOY_BIN="$AHOY_HOME/bin/ahoy"

echo "Uninstalling Ahoy..."
echo ""

# Remove Claude Code hooks if installed
if [ -f "$AHOY_BIN" ]; then
    echo "Removing Claude Code hooks..."
    "$AHOY_BIN" uninstall claude 2>/dev/null || true
fi

# Unregister from macOS Launch Services
if [[ "$OSTYPE" == "darwin"* ]] && [ -d "$AHOY_HOME/Ahoy.app" ]; then
    echo "Unregistering from macOS Launch Services..."
    /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -u "$AHOY_HOME/Ahoy.app" 2>/dev/null || true
fi

# Remove ahoy directory
if [ -d "$AHOY_HOME" ]; then
    echo "Removing $AHOY_HOME..."
    rm -rf "$AHOY_HOME"
    echo "✓ Ahoy uninstalled"
else
    echo "Ahoy is not installed at $AHOY_HOME"
fi

echo ""
echo "Note: You may want to remove this line from your shell config:"
echo "  export PATH=\"\$HOME/.ahoy/bin:\$PATH\""
echo ""
