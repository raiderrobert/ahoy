#!/bin/sh
# Ahoy installer — https://github.com/raiderrobert/ahoy
# Usage: curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/install.sh | sh
set -e

REPO="raiderrobert/ahoy"
AHOY_HOME="${AHOY_HOME:-$HOME/.ahoy}"
AHOY_BIN="$AHOY_HOME/bin"
AHOY_APP="$AHOY_HOME/Ahoy.app"

main() {
    platform="$(detect_platform)"
    arch="$(detect_arch)"
    asset="$(asset_name "$platform" "$arch")"

    if [ -z "$asset" ]; then
        echo "Error: unsupported platform/architecture: ${platform}/${arch}" >&2
        echo "Pre-built binaries are available for:" >&2
        echo "  - macOS (Apple Silicon / aarch64)" >&2
        exit 1
    fi

    url="https://github.com/${REPO}/releases/latest/download/${asset}"

    echo "Installing Ahoy - notification CLI for LLM coding agents"
    echo ""
    echo "Detected: ${platform}/${arch}"
    echo "Downloading: ${url}"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$url" -o "${tmpdir}/${asset}"
    elif command -v wget > /dev/null 2>&1; then
        wget -qO "${tmpdir}/${asset}" "$url"
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi

    tar xzf "${tmpdir}/${asset}" -C "$tmpdir"

    # Install binary
    mkdir -p "$AHOY_BIN"
    cp "${tmpdir}/ahoy/ahoy" "$AHOY_BIN/ahoy"
    chmod +x "$AHOY_BIN/ahoy"

    # Install Ahoy.app bundle
    rm -rf "$AHOY_APP"
    cp -R "${tmpdir}/ahoy/Ahoy.app" "$AHOY_APP"

    # macOS post-install: remove quarantine, code sign, register with Launch Services
    xattr -cr "$AHOY_BIN/ahoy" 2>/dev/null || true
    xattr -cr "$AHOY_APP" 2>/dev/null || true
    codesign -s - "$AHOY_BIN/ahoy" 2>/dev/null || true
    codesign -s - "$AHOY_APP/Contents/MacOS/ahoy-notify" 2>/dev/null || true
    /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$AHOY_APP"

    echo ""
    echo "Ahoy installed successfully!"
    echo ""
    echo "Binary location: $AHOY_BIN/ahoy"
    echo ""

    # Check if ahoy is in PATH
    if ! echo ":$PATH:" | grep -q ":$AHOY_BIN:"; then
        echo "To add ahoy to your PATH, add this to your shell config:"
        echo ""
        echo "  export PATH=\"\$HOME/.ahoy/bin:\$PATH\""
        echo ""
    fi

    # Point user to Claude Code plugin for hook installation
    echo "To set up Claude Code notifications, run these inside Claude Code:"
    echo ""
    echo "  /plugin marketplace add raiderrobert/ahoy"
    echo "  /plugin install ahoy-hooks@ahoy"
    echo ""
    echo "Or install hooks manually with: $AHOY_BIN/ahoy install claude"
    echo ""
    echo "Test it with: $AHOY_BIN/ahoy send 'Hello from Ahoy!'"
    echo ""
}

detect_platform() {
    case "$(uname -s)" in
        Darwin*) echo "macos" ;;
        *)       echo "unknown" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *)             echo "unknown" ;;
    esac
}

asset_name() {
    platform="$1"
    arch="$2"

    case "${arch}-${platform}" in
        aarch64-macos) echo "ahoy-aarch64-macos.tar.gz" ;;
        *)             echo "" ;;
    esac
}

main
