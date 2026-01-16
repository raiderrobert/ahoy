# Ahoy

Desktop notifications for LLM coding agents.

When Claude Code (or other AI coding tools) finishes a task, needs your input, or requires permission, Ahoy shows a native macOS notification so you know it's time to check in.

## Quick Install

```bash
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/install.sh | bash
```

Or clone and install manually:

```bash
git clone https://github.com/raiderrobert/ahoy.git
cd ahoy
./install.sh
```

The installer will:
1. Build the Rust CLI and Swift notification helper
2. Install to `~/.ahoy/`
3. Prompt you to install Claude Code hooks

**Add to PATH:**
```bash
export PATH="$HOME/.ahoy/bin:$PATH"
```

## Usage

### Send notifications manually

```bash
ahoy send "Task completed"                      # Simple notification
ahoy send -t "Custom Title" "Message here"      # Custom title
ahoy send --activate com.apple.Terminal "Done"  # Focus Terminal when clicked
```

### Claude Code integration

There are two ways to set up Claude Code notifications:

#### Option 1: CLI install (recommended)

```bash
ahoy install claude
```

This adds hooks to `~/.claude/settings.json` that trigger notifications when:
- **Stop**: Claude finishes a task (shows the last user prompt)
- **Idle prompt**: Claude is waiting for your input
- **Permission prompt**: Claude needs permission to proceed

To remove hooks:

```bash
ahoy uninstall claude
```

#### Option 2: Claude Code plugin

If you prefer using the Claude Code plugin system:

```bash
# First, install the ahoy binary
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/install.sh | bash

# Then install the hooks plugin in Claude Code
/plugin marketplace add raiderrobert/ahoy
/plugin install ahoy-hooks@ahoy-hooks
```

The plugin installs the same hooks as the CLI method.

Clicking a notification will bring your terminal to the front.

## How it works

1. `ahoy send` calls the Swift notification helper directly (no daemon)
2. Hooks in `~/.claude/settings.json` run `ahoy send` at key moments
3. The `--from-claude` flag extracts your last prompt from stdin
4. The `--activate` flag focuses your terminal when you click the notification
5. macOS shows a native notification with sound

## Requirements

- macOS (Linux/Windows support planned)
- Rust toolchain (auto-installed by install script)
- Swift / Xcode Command Line Tools: `xcode-select --install`

## Commands

```bash
ahoy send [OPTIONS] [MESSAGE]    # Send a notification
ahoy install claude              # Install Claude Code hooks
ahoy uninstall claude            # Remove Claude Code hooks
ahoy --help                      # Show all options
```

## Advanced Options

```bash
# Read Claude Code hook data from stdin to extract last prompt
ahoy send --from-claude -t "Title" --activate "$__CFBundleIdentifier"

# Send custom JSON payload
ahoy send --json '{"title":"Custom","body":"Message","activate":"com.app.id"}'
```

## Uninstall

```bash
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/uninstall.sh | bash
```

Or manually:

```bash
ahoy uninstall claude  # Remove hooks first
rm -rf ~/.ahoy
```

Don't forget to remove the PATH export from your shell config.
