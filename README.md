# ahoy

Native macOS notifications for AI coding agents.

- No need to sit and babysit. Get pinged when your agent finishes, gets stuck, or needs permission.
- Simple UX. Just 3 kind of notifications: done, waiting for input, waiting for permission. Click the notification to refocus. Your terminal comes to the front.
- Simple design. There's no background daemon or process. It simply pushes events via your agent's hooks to the notification center in macOS.

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

```bash
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/install.sh | sh
```

Then inside Claude Code:

```
/plugin marketplace add raiderrobert/ahoy
/plugin install ahoy-hooks@ahoy
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
ahoy --help                      Show all options
```

## Requirements

- macOS
- Xcode Command Line Tools: `xcode-select --install`

## Uninstall

```bash
curl -sSL https://raw.githubusercontent.com/raiderrobert/ahoy/main/uninstall.sh | bash
```

Or manually: `rm -rf ~/.ahoy`
