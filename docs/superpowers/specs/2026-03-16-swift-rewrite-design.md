# Ahoy Swift Rewrite — Design Spec

## Summary

Collapse the current two-binary architecture (Rust CLI + Swift notification helper) into a single Swift binary. The Rust crate is deleted entirely. The result is one file (`swift/ahoy.swift`), one binary (`ahoy`), living inside the existing `Ahoy.app` bundle.

## Motivation

- The Rust binary's only job is CLI parsing and Claude hook stdin processing, then it shells out to the Swift binary for the actual notification. This two-process hop adds complexity for no benefit.
- Ahoy is macOS-only. There is no cross-platform story to protect.
- A single Swift binary simplifies the build, release, and install pipeline.

## Architecture

One Swift source file compiled with `swiftc`. No SwiftPM, no package manager, no dependencies beyond Foundation and AppKit (same as today).

```
Claude Code hook → stdin JSON → ahoy (parses, formats, shows notification)
                                  or: ahoy "message" --title "Title" (direct mode)
```

The binary lives at `Ahoy.app/Contents/MacOS/ahoy`. At install time, `~/.ahoy/bin/ahoy` is a symlink or copy pointing there.

### Notification API

Uses `NSUserNotificationCenter` (deprecated since macOS 10.14, still functional through macOS 15). Deployment target is macOS 13.0. No migration to `UNUserNotificationCenter` — that requires permission prompts and more app ceremony. If Apple removes the deprecated API in a future macOS release, migrating is a scoped change in one file.

### Notification Lifecycle

The binary must stay alive after delivering the notification:
- **With `--activate`**: Run an `NSApplication` RunLoop for up to 60 seconds, waiting for the user to click the notification. On click, activate the target app and exit.
- **Without `--activate`**: Run the RunLoop for 0.5 seconds so macOS has time to display the notification, then exit.

This lifecycle is inherited from the existing Swift code and must be preserved. `NSApplication.shared` is initialized with `.accessory` activation policy (no dock icon, no menu bar).

### Focus Check

Before showing a notification, if `--activate` is set, check whether the target app is already the frontmost application. If so, skip the notification silently (the user is already looking at it). Exit 0.

## CLI Interface

Flat, no subcommands. The binary does one thing: send a notification.

```
ahoy [message] [options]

Arguments:
  message              Notification body text (optional if --from-claude or --json)

Options:
  -t, --title <text>   Notification title (default: "Ahoy")
  --json <json>        Raw JSON: {"title":"...","body":"...","activate":"..."}
  --from-claude        Read Claude Code hook data from stdin
  --activate <id>      Bundle ID to activate on click
  --sound <name>       Notification sound (default: "Glass")
  --help               Show usage
```

Changes from current Rust CLI:
- `send` subcommand removed (was the only subcommand)
- `--sound` exposed as a flag (was hardcoded)
- `--help` is hand-rolled (usage print + exit)
- `--version` dropped (no longer have Cargo.toml to source it; not worth the complexity)
- `--icon` dropped (unused; icon is handled by bundle swizzling)

## Internal Structure

Single file organized with `// MARK:` sections:

```
// MARK: - Data Types
struct Notification (title, body, activate, sound)
struct ClaudeHookData (transcript_path, cwd, tool_name, tool_input, session_id, hook_event_name)

// MARK: - CLI Parsing
parseArgs() → (message?, title, json?, fromClaude, activate?, sound)

// MARK: - Hook Processing
buildFromClaudeStdin() → Notification
extractLastPrompt(transcriptPath) → String?
extractProjectName(cwd) → String

// MARK: - Notification Display
Bundle swizzling (existing code, unchanged)
NotificationDelegate class (existing code, unchanged)
showNotification(Notification) → delivers via NSUserNotificationCenter

// MARK: - Main
Parse args → build Notification → focus check → show
```

### Bug Fixes Included

- **UTF-8 safe truncation**: Uses `String.prefix()` instead of byte slicing (the Rust code panics on multi-byte characters at truncation boundaries)
- **Trailing slash in cwd**: Trims trailing slashes before extracting project name (currently produces `[]` instead of `[projectname]`)
- **Error log level**: Notification failures go to stderr as errors (Rust code logged them at info level)

### JSON Parsing

Uses Foundation's `Codable` protocol — built in, zero dependencies. `ClaudeHookData` and the transcript line format are decoded with `JSONDecoder`.

**Important:** Swift's `JSONDecoder` rejects unknown keys by default (unlike Rust's serde). All `Codable` structs must either list every possible field from the JSON as optional properties, or use `CodingKeys` with a custom `init(from:)` that ignores unknown keys. The safest approach: make all fields optional and list the full set.

### Intentionally Dropped Fields

The Rust `Notification` struct had `icon` and `metadata` fields. These are **dropped** in the rewrite:
- `icon` was never passed to the Swift binary — dead plumbing
- `metadata` (`HashMap<String, Value>`) was never consumed anywhere

The `--icon` flag from the current `ahoy-notify` binary is also dropped — the app icon is handled entirely by bundle swizzling, and the `--icon` flag was unused in practice.

The `--json` mode accepts `{"title":"...","body":"..."}` where `title` and `body` are required. Optional fields: `activate`, `sound`. Unknown fields are ignored.

### Empty vs Invalid Stdin

When `--from-claude` is set:
- **Empty stdin** (zero bytes): return a fallback notification with body "Task finished". Exit 0.
- **Non-empty but invalid JSON**: print error to stderr. Exit non-zero.

This matches the current Rust behavior. The check is on byte length, not whitespace.

## Files Changed

### Deleted

- `src/` — all Rust source code
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`
- `tests/` — Rust test fixtures (replaced by integration tests)

### Renamed / Moved

- `swift/ahoy-notify.swift` → `swift/ahoy.swift`
- Binary output name: `ahoy-notify` → `ahoy`

### Updated

- `swift/Makefile` — new binary name, updated build/install targets
- `swift/Info.plist` — executable name `ahoy-notify` → `ahoy`
- `hooks/hooks.json` — drop `send` from all hook commands
- `install.sh` — no separate Rust binary install; `~/.ahoy/bin/ahoy` symlinks to `Ahoy.app/Contents/MacOS/ahoy`
- `uninstall.sh` — simplified
- `.github/workflows/ci.yml` — remove Rust steps, Swift build + integration tests only
- `.github/workflows/release.yml` — remove Rust build, archive is `Ahoy.app` + scripts
- `justfile` — recipes become Swift-oriented
- `release-please-config.json` — release type from `rust` to `simple`; preserve extra-files config for `marketplace.json` and `plugin.json` version patching

### New

- `tests/integration.sh` — shell-based integration tests

### Unchanged

- `Ahoy.app/Contents/Resources/` — icons
- `README.md` — minor cosmetic updates only

## Testing Strategy

Shell-based integration tests that exercise the real binary. Run via `make test` in the Makefile. CI runs the same.

### Test Cases

1. **Direct message** — `ahoy "Hello" -t "Test"` → exits 0
2. **JSON input** — `ahoy --json '{"title":"T","body":"B"}'` → exits 0
3. **`--from-claude` with tool permission** — pipe tool data to stdin → exits 0
4. **`--from-claude` with transcript** — create temp JSONL, pipe hook data → exits 0
5. **`--from-claude` empty stdin** — pipe empty string → exits 0, falls back to "Task finished"
6. **`--from-claude` invalid JSON** — pipe garbage → exits non-zero
7. **No message and no flags** — exits non-zero
8. **`--help`** — exits 0, output contains "Usage"
9. **Trailing slash cwd** — pipe `{"cwd":"/foo/bar/"}` → body contains `[bar]` not `[]`

## Hook Commands

All hooks drop `send` from the command string. The existing `hooks.json` structure (nested hooks array, matchers, timeouts) is preserved — only the `command` values change:

- **Stop**: `$HOME/.ahoy/bin/ahoy --from-claude -t 'Claude Code' --activate "$__CFBundleIdentifier"`
- **Notification (idle_prompt)**: `$HOME/.ahoy/bin/ahoy -t 'Claude Code' 'Waiting for your input' --activate "$__CFBundleIdentifier"`
- **Notification (permission_prompt)**: `$HOME/.ahoy/bin/ahoy --from-claude -t 'Claude Code' --activate "$__CFBundleIdentifier"`
```
