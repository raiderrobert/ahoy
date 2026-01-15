#!/bin/bash
set -e

# Ahoy installer - builds from source
# Usage: curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/install.sh | bash

AHOY_HOME="$HOME/.ahoy"
AHOY_BIN="$AHOY_HOME/bin"
AHOY_APP="$AHOY_HOME/Ahoy.app"
REPO_URL="https://github.com/raiderrobert/ahoy.git"

echo "Installing Ahoy - notification CLI for LLM coding agents"
echo ""

# Check for required tools
check_rust() {
    if command -v cargo &> /dev/null; then
        echo "✓ Rust is installed"
        return 0
    else
        echo "Rust not found. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo "✓ Rust installed"
    fi
}

check_swift() {
    if command -v swiftc &> /dev/null; then
        echo "✓ Swift is installed"
        return 0
    else
        echo "Error: Swift compiler (swiftc) not found."
        echo "Please install Xcode Command Line Tools: xcode-select --install"
        exit 1
    fi
}

# Create temp directory for build
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo ""
echo "Checking dependencies..."
check_rust
check_swift

echo ""
echo "Cloning repository..."
git clone --depth 1 "$REPO_URL" "$TEMP_DIR/ahoy"
cd "$TEMP_DIR/ahoy"

echo ""
echo "Building Rust binary..."
cargo build --release

echo ""
echo "Building Swift helper..."
swiftc -O -o Ahoy.app/Contents/MacOS/ahoy-notify swift/ahoy-notify.swift

echo ""
echo "Installing to $AHOY_HOME..."

# Create directory structure
mkdir -p "$AHOY_BIN"

# Copy binary
cp target/release/ahoy "$AHOY_BIN/ahoy"

# Copy Ahoy.app bundle (contains Swift helper, icons, Info.plist)
rm -rf "$AHOY_APP"
cp -R Ahoy.app "$AHOY_APP"

# Remove quarantine attributes and code sign (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Code signing binaries..."
    xattr -cr "$AHOY_BIN/ahoy" 2>/dev/null || true
    xattr -cr "$AHOY_APP" 2>/dev/null || true
    codesign -s - "$AHOY_BIN/ahoy" 2>/dev/null || true
    codesign -s - "$AHOY_APP/Contents/MacOS/ahoy-notify" 2>/dev/null || true
fi

echo ""
echo "✓ Ahoy installed successfully!"
echo ""
echo "Binary location: $AHOY_BIN/ahoy"
echo ""

# Check if ahoy is in PATH
if [[ ":$PATH:" != *":$AHOY_BIN:"* ]]; then
    echo "To add ahoy to your PATH, add this to your shell config:"
    echo ""
    echo "  export PATH=\"\$HOME/.ahoy/bin:\$PATH\""
    echo ""
fi

# Install Claude Code hooks if running interactively
if [ -t 0 ]; then
    echo "Would you like to install Claude Code hooks? (y/n)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        "$AHOY_BIN/ahoy" install claude
        echo ""
        echo "✓ Claude Code hooks installed!"
    fi
else
    echo "To install Claude Code hooks, run:"
    echo "  $AHOY_BIN/ahoy install claude"
fi

echo ""
echo "Test it with: $AHOY_BIN/ahoy send 'Hello from Ahoy!'"
echo ""
