# ahoy

Native macOS notifications for AI coding agents.

- **Walk away from long tasks.** Get pinged when your agent finishes, gets stuck, or needs permission.
- **Signal, not noise.** Three events, nothing else — done, waiting for input, waiting for permission.
- **Click to refocus.** Tap the notification and your terminal comes to the front.
- **No daemon, no polling.** Hooks call a CLI binary on demand. Zero background overhead.

## Install

```bash
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/install.sh | bash
```

Or build from source:

```bash
git clone https://github.com/raiderrobert/ahoy.git
cd ahoy
./install.sh
```

Add to PATH:

```bash
export PATH="$HOME/.ahoy/bin:$PATH"
```

## Claude Code Setup

### Plugin (recommended)

```bash
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/install.sh | bash
```

Then inside Claude Code:

```
/plugin marketplace add raiderrobert/ahoy
/plugin install ahoy-hooks@ahoy
```

### Manual hooks

```bash
ahoy install claude       # writes hooks to ~/.claude/settings.json
ahoy uninstall claude     # removes them
```

## Usage

```bash
ahoy send "Task completed"                      # Simple notification
ahoy send -t "Custom Title" "Message here"      # Custom title
ahoy send --activate com.apple.Terminal "Done"   # Focus Terminal when clicked
```

## Commands

```
ahoy send [OPTIONS] [MESSAGE]    Send a notification
ahoy install claude              Install Claude Code hooks
ahoy uninstall claude            Remove Claude Code hooks
ahoy --help                      Show all options
```

## Requirements

- macOS
- Xcode Command Line Tools: `xcode-select --install`

## Uninstall

```bash
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/uninstall.sh | bash
```

Or manually:

```bash
ahoy uninstall claude  # Remove hooks first
rm -rf ~/.ahoy
```
