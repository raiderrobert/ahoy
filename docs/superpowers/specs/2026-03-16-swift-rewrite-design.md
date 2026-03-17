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

Uses `NSUserNotificationCenter` (deprecated but functional). No migration to `UNUserNotificationCenter` — that requires permission prompts and more app ceremony. If Apple removes the deprecated API in a future macOS release, migrating is a scoped change in one file.

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

## Internal Structure

Single file organized with `// MARK:` sections:

```
// MARK: - Data Types
struct Notification (title, body, activate, sound)
struct ClaudeHookData (transcript_path, cwd, tool_name, tool_input)

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
- `release-please-config.json` — release type from `rust` to `simple`

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

All hooks drop `send`:

```json
{
  "Stop": "$HOME/.ahoy/bin/ahoy --from-claude -t 'Claude Code' --activate \"$__CFBundleIdentifier\"",
  "Notification (idle)": "$HOME/.ahoy/bin/ahoy -t 'Claude Code' 'Waiting for your input' --activate \"$__CFBundleIdentifier\"",
  "Notification (permission)": "$HOME/.ahoy/bin/ahoy --from-claude -t 'Claude Code' --activate \"$__CFBundleIdentifier\""
}
```
