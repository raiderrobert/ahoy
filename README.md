# Ahoy

Desktop notifications for LLM coding agents.

When Claude Code (or other AI coding tools) finishes a task, Ahoy shows a native notification so you know it's done.

## Install

```bash
# Build and install
cargo build --release
cp target/release/ahoy ~/.ahoy/bin/

# Install the Swift notification helper (macOS)
cd swift && make install

# Start the daemon
ahoy service install
ahoy service start

# Add Claude Code hook
ahoy install claude
```

## Usage

```bash
ahoy send "Task completed"           # Quick notification
ahoy send -t "Title" "Message"       # With custom title
ahoy status                          # Check daemon status
ahoy service restart                 # Restart daemon
```

## How it works

1. Ahoy daemon listens on `~/.ahoy/ahoy.sock`
2. Claude Code's Stop hook calls `ahoy send` when tasks finish
3. Native macOS notification appears with sound

## Requirements

- macOS (Linux/Windows support planned)
- Rust toolchain
- Swift (for macOS notifications)
