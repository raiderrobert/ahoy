# Swift Rewrite Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Rust CLI + Swift helper with a single Swift binary that handles CLI parsing, Claude hook processing, and native macOS notifications.

**Architecture:** One Swift file (`swift/ahoy.swift`) compiled with `swiftc`, living inside `Ahoy.app/Contents/MacOS/ahoy`. No package manager. Hand-rolled arg parsing. `NSUserNotificationCenter` for notifications.

**Tech Stack:** Swift, Foundation, AppKit, `swiftc` (no SwiftPM)

**Spec:** `docs/superpowers/specs/2026-03-16-swift-rewrite-design.md`

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `swift/ahoy.swift` | All logic: data types, CLI parsing, hook processing, notification display |
| Create | `tests/integration.sh` | Shell-based integration tests |
| Modify | `swift/Makefile` | Updated binary name, build targets, test target |
| Modify | `swift/Info.plist` | Executable name → `ahoy` |
| Modify | `hooks/hooks.json` | Drop `send` from commands |
| Modify | `install.sh` | Symlink instead of separate binary |
| Modify | `uninstall.sh` | No changes needed (already removes `~/.ahoy/`) |
| Modify | `.github/workflows/ci.yml` | Remove Rust, add integration tests |
| Modify | `.github/workflows/release.yml` | Remove Rust build |
| Modify | `justfile` | Swift-oriented recipes |
| Modify | `release-please-config.json` | `rust` → `simple` |
| Delete | `src/` | All Rust source |
| Delete | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | Rust build config |
| Delete | `tests/fixtures/`, `tests/CLAUDE.md` | Rust test fixtures (replaced by `tests/integration.sh`) |
| Delete | `swift/ahoy-notify.swift` | Replaced by `swift/ahoy.swift` |
| Modify | `swift/.gitignore` | Ignore `ahoy` instead of `ahoy-notify` |
| Delete | `lefthook.yml` | Rust-specific pre-commit hook (cargo fmt) |
| Create | `version.txt` | Version file for `simple` release-please type |

---

### Task 1: Write the Swift binary

**Files:**
- Create: `swift/ahoy.swift`
- Reference: `swift/ahoy-notify.swift` (existing notification code to carry over)
- Reference: `src/client/send.rs` (hook processing logic to port)
- Reference: `src/client/message.rs` (Notification struct to port)

- [ ] **Step 1: Create `swift/ahoy.swift` with data types**

```swift
#!/usr/bin/env swift

import Foundation
import AppKit
import ObjectiveC

// MARK: - Data Types

struct AhoyNotification {
    let title: String
    let body: String
    var activate: String?
    var sound: String
}

struct ClaudeHookData: Codable {
    let transcriptPath: String?
    let cwd: String?
    let toolName: String?
    let toolInput: [String: AnyCodableValue]?

    enum CodingKeys: String, CodingKey {
        case transcriptPath = "transcript_path"
        case cwd
        case toolName = "tool_name"
        case toolInput = "tool_input"
    }
}

// Lightweight wrapper to decode arbitrary JSON values from tool_input.
// Only scalar types are supported — nested objects/arrays decode as .null.
// This is intentional: we only access .stringValue for command/file_path/pattern.
enum AnyCodableValue: Codable {
    case string(String)
    case int(Int)
    case double(Double)
    case bool(Bool)
    case null

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let s = try? container.decode(String.self) { self = .string(s) }
        else if let i = try? container.decode(Int.self) { self = .int(i) }
        else if let d = try? container.decode(Double.self) { self = .double(d) }
        else if let b = try? container.decode(Bool.self) { self = .bool(b) }
        else if container.decodeNil() { self = .null }
        else { self = .null }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let s): try container.encode(s)
        case .int(let i): try container.encode(i)
        case .double(let d): try container.encode(d)
        case .bool(let b): try container.encode(b)
        case .null: try container.encodeNil()
        }
    }

    var stringValue: String? {
        if case .string(let s) = self { return s }
        return nil
    }
}

struct TranscriptLine: Codable {
    let type: String?
    let message: TranscriptMessage?
}

struct TranscriptMessage: Codable {
    let content: TranscriptContent?
}

enum TranscriptContent: Codable {
    case string(String)
    case array([[String: String]])

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let s = try? container.decode(String.self) {
            self = .string(s)
        } else if let arr = try? container.decode([[String: String]].self) {
            self = .array(arr)
        } else {
            self = .string("")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let s): try container.encode(s)
        case .array(let arr): try container.encode(arr)
        }
    }
}
```

- [ ] **Step 2: Add CLI parsing**

Append to `swift/ahoy.swift`:

```swift
// MARK: - CLI Parsing

struct ParsedArgs {
    var message: String?
    var title: String = "Ahoy"
    var json: String?
    var fromClaude: Bool = false
    var activate: String?
    var sound: String = "Glass"
}

func printUsage() {
    fputs("""
    Usage: ahoy [message] [options]

    Arguments:
      message              Notification body text (optional if --from-claude or --json)

    Options:
      -t, --title <text>   Notification title (default: "Ahoy")
      --json <json>        Raw JSON notification
      --from-claude        Read Claude Code hook data from stdin
      --activate <id>      Bundle ID to activate on click
      --sound <name>       Notification sound (default: "Glass")
      --help               Show this help message

    """, stderr)
}

func parseArgs() -> ParsedArgs {
    var parsed = ParsedArgs()
    let args = CommandLine.arguments
    var i = 1

    while i < args.count {
        switch args[i] {
        case "--help", "-h":
            printUsage()
            exit(0)
        case "-t", "--title":
            guard i + 1 < args.count else {
                fputs("Error: --title requires a value\n", stderr)
                exit(1)
            }
            parsed.title = args[i + 1]
            i += 2
        case "--json":
            guard i + 1 < args.count else {
                fputs("Error: --json requires a value\n", stderr)
                exit(1)
            }
            parsed.json = args[i + 1]
            i += 2
        case "--from-claude":
            parsed.fromClaude = true
            i += 1
        case "--activate":
            guard i + 1 < args.count else {
                fputs("Error: --activate requires a value\n", stderr)
                exit(1)
            }
            parsed.activate = args[i + 1]
            i += 2
        case "--sound":
            guard i + 1 < args.count else {
                fputs("Error: --sound requires a value\n", stderr)
                exit(1)
            }
            parsed.sound = args[i + 1]
            i += 2
        default:
            if args[i].hasPrefix("-") {
                fputs("Error: unknown option '\(args[i])'\n", stderr)
                printUsage()
                exit(1)
            }
            if parsed.message == nil {
                parsed.message = args[i]
            }
            i += 1
        }
    }

    return parsed
}
```

- [ ] **Step 3: Add hook processing**

Append to `swift/ahoy.swift`:

```swift
// MARK: - Hook Processing

func extractProjectName(from cwd: String?) -> String {
    guard let cwd = cwd else { return "project" }
    let trimmed = cwd.hasSuffix("/") ? String(cwd.dropLast()) : cwd
    return trimmed.split(separator: "/").last.map(String.init) ?? "project"
}

func truncate(_ s: String, to maxLength: Int) -> String {
    if s.count > maxLength {
        return String(s.prefix(maxLength - 3)) + "..."
    }
    return s
}

func extractLastPrompt(from transcriptPath: String) -> String? {
    guard let data = FileManager.default.contents(atPath: transcriptPath),
          let contents = String(data: data, encoding: .utf8) else {
        return nil
    }

    let decoder = JSONDecoder()
    var lastUserContent: String? = nil

    for line in contents.components(separatedBy: .newlines) {
        guard !line.isEmpty,
              let lineData = line.data(using: .utf8),
              let entry = try? decoder.decode(TranscriptLine.self, from: lineData),
              entry.type == "user",
              let message = entry.message,
              let content = message.content else {
            continue
        }

        let text: String
        switch content {
        case .string(let s):
            text = s
        case .array(let arr):
            text = arr.compactMap { $0["text"] }.joined(separator: " ")
        }

        let cleaned = text.components(separatedBy: .newlines).first?.trimmingCharacters(in: .whitespaces) ?? text.trimmingCharacters(in: .whitespaces)

        if !cleaned.isEmpty {
            lastUserContent = cleaned
        }
    }

    return lastUserContent
}

func buildFromClaudeStdin(title: String) -> AhoyNotification {
    let stdinData = FileHandle.standardInput.readDataToEndOfFile()

    guard !stdinData.isEmpty else {
        return AhoyNotification(title: title, body: "Task finished", sound: "Glass")
    }

    guard let hookData = try? JSONDecoder().decode(ClaudeHookData.self, from: stdinData) else {
        fputs("Error: failed to parse Claude hook data from stdin\n", stderr)
        exit(1)
    }

    let projectName = extractProjectName(from: hookData.cwd)

    // If there's a tool_name, this is a permission prompt
    if let toolName = hookData.toolName {
        let toolDesc: String
        if let input = hookData.toolInput {
            let raw = input["command"]?.stringValue
                ?? input["file_path"]?.stringValue
                ?? input["pattern"]?.stringValue
            toolDesc = raw.map { truncate($0, to: 60) } ?? ""
        } else {
            toolDesc = ""
        }

        let body: String
        if toolDesc.isEmpty {
            body = "[\(projectName)] Needs permission: \(toolName)"
        } else {
            body = "[\(projectName)] \(toolName): \(toolDesc)"
        }

        return AhoyNotification(title: title, body: body, sound: "Glass")
    }

    // Otherwise, extract the last user prompt from transcript
    let lastPrompt: String
    if let transcriptPath = hookData.transcriptPath {
        lastPrompt = extractLastPrompt(from: transcriptPath) ?? "Task finished"
    } else {
        lastPrompt = "Task finished"
    }

    let body = "[\(projectName)] \(truncate(lastPrompt, to: 100))"
    return AhoyNotification(title: title, body: body, sound: "Glass")
}
```

- [ ] **Step 4: Add notification display (carry over from ahoy-notify.swift)**

Append to `swift/ahoy.swift`:

```swift
// MARK: - Bundle Identifier Swizzling

let fakeBundleIdentifier = "rs.ahoy.notify.fresh"

func installFakeBundleIdentifierHook() {
    guard let bundleClass = objc_getClass("NSBundle") as? AnyClass else {
        fputs("Failed to get NSBundle class\n", stderr)
        return
    }

    let originalSelector = NSSelectorFromString("bundleIdentifier")
    guard let originalMethod = class_getInstanceMethod(bundleClass, originalSelector) else {
        fputs("Failed to get bundleIdentifier method\n", stderr)
        return
    }

    let originalImp = method_getImplementation(originalMethod)
    typealias OriginalFunc = @convention(c) (AnyObject, Selector) -> String?
    let original: OriginalFunc = unsafeBitCast(originalImp, to: OriginalFunc.self)

    let newImp: @convention(block) (AnyObject) -> String? = { (self) in
        if self === Bundle.main {
            return fakeBundleIdentifier
        }
        return original(self, originalSelector)
    }

    method_setImplementation(originalMethod, imp_implementationWithBlock(newImp))
}

// MARK: - Notification Delegate

class NotificationDelegate: NSObject, NSUserNotificationCenterDelegate {
    var activateBundleId: String?
    var didActivate = false

    func userNotificationCenter(_ center: NSUserNotificationCenter, didActivate notification: NSUserNotification) {
        didActivate = true
        if let bundleId = activateBundleId {
            let runningApps = NSWorkspace.shared.runningApplications.filter { $0.bundleIdentifier == bundleId }
            if let app = runningApps.first {
                app.activate()
            } else {
                let task = Process()
                task.launchPath = "/usr/bin/open"
                task.arguments = ["-b", bundleId]
                try? task.run()
            }
        }
        exit(0)
    }

    func userNotificationCenter(_ center: NSUserNotificationCenter, shouldPresent notification: NSUserNotification) -> Bool {
        return true
    }
}

// MARK: - Show Notification

func showNotification(_ notification: AhoyNotification, delegate: NotificationDelegate) {
    let nsNotification = NSUserNotification()
    nsNotification.title = notification.title
    nsNotification.informativeText = notification.body
    nsNotification.soundName = notification.sound

    NSUserNotificationCenter.default.deliver(nsNotification)
    fputs("Notification delivered\n", stderr)

    // Keep alive for click handling or delivery
    if notification.activate != nil {
        let timeout = Date(timeIntervalSinceNow: 60)
        while !delegate.didActivate && Date() < timeout {
            RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.1))
        }
    } else {
        RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.5))
    }
}
```

- [ ] **Step 5: Add main entry point**

Append to `swift/ahoy.swift`:

```swift
// MARK: - Main

installFakeBundleIdentifierHook()

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

let notificationDelegate = NotificationDelegate()
NSUserNotificationCenter.default.delegate = notificationDelegate

let parsed = parseArgs()

// Build notification (precedence: --from-claude > --json > positional message)
var notification: AhoyNotification

if parsed.fromClaude {
    notification = buildFromClaudeStdin(title: parsed.title)
    notification.sound = parsed.sound
} else if let jsonStr = parsed.json {
    guard let jsonData = jsonStr.data(using: .utf8) else {
        fputs("Error: invalid JSON string\n", stderr)
        exit(1)
    }

    struct JsonNotification: Codable {
        let title: String
        let body: String
        let activate: String?
        let sound: String?
    }

    guard let jsonNotif = try? JSONDecoder().decode(JsonNotification.self, from: jsonData) else {
        fputs("Error: failed to parse JSON notification\n", stderr)
        exit(1)
    }

    notification = AhoyNotification(
        title: jsonNotif.title,
        body: jsonNotif.body,
        activate: jsonNotif.activate,
        sound: jsonNotif.sound ?? parsed.sound
    )
} else if let message = parsed.message {
    notification = AhoyNotification(title: parsed.title, body: message, sound: parsed.sound)
} else {
    fputs("Error: a message, --json, or --from-claude is required\n", stderr)
    printUsage()
    exit(1)
}

// Apply --activate override (overrides any value from JSON/stdin)
if let activate = parsed.activate {
    notification.activate = activate
}

// Focus check: skip if target app is already frontmost
if let bundleId = notification.activate {
    if let frontmost = NSWorkspace.shared.frontmostApplication,
       frontmost.bundleIdentifier == bundleId {
        fputs("Target app is focused (\(bundleId)), skipping notification\n", stderr)
        exit(0)
    }
}

notificationDelegate.activateBundleId = notification.activate
showNotification(notification, delegate: notificationDelegate)
```

- [ ] **Step 6: Verify it compiles**

Run: `cd swift && swiftc -O -target arm64-apple-macos13.0 -o ahoy ahoy.swift -Xlinker -sectcreate -Xlinker __TEXT -Xlinker __info_plist -Xlinker Info.plist`
Expected: Compiles with no errors (deprecation warnings for `NSUserNotification` are expected and OK)

- [ ] **Step 7: Quick manual smoke test**

Run: `swift/ahoy "Hello from Swift" -t "Test"`
Expected: macOS notification appears with title "Test" and body "Hello from Swift"

- [ ] **Step 8: Commit**

```bash
git add swift/ahoy.swift
git commit -m "feat: rewrite CLI in Swift — single binary replaces Rust+Swift"
```

---

### Task 2: Write integration tests

**Files:**
- Create: `tests/integration.sh`

- [ ] **Step 1: Create `tests/integration.sh`**

```bash
#!/bin/bash
set -euo pipefail

AHOY="${1:?Usage: integration.sh <path-to-ahoy-binary>}"
PASS=0
FAIL=0

pass() { PASS=$((PASS + 1)); echo "  PASS: $1"; }
fail() { FAIL=$((FAIL + 1)); echo "  FAIL: $1 — $2"; }

echo "Running ahoy integration tests..."
echo ""

# 1. Direct message
if "$AHOY" "Hello" -t "Test" 2>/dev/null; then
    pass "direct message"
else
    fail "direct message" "non-zero exit"
fi

# 2. JSON input
if "$AHOY" --json '{"title":"T","body":"B"}' 2>/dev/null; then
    pass "json input"
else
    fail "json input" "non-zero exit"
fi

# 3. --from-claude with tool permission
TOOL_JSON='{"cwd":"/Users/test/myproject","tool_name":"Bash","tool_input":{"command":"npm install"}}'
if echo "$TOOL_JSON" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "from-claude tool permission"
else
    fail "from-claude tool permission" "non-zero exit"
fi

# 4. --from-claude with transcript
TMPDIR_TESTS=$(mktemp -d)
trap 'rm -rf "$TMPDIR_TESTS"' EXIT
TRANSCRIPT="$TMPDIR_TESTS/transcript.jsonl"
echo '{"type":"user","message":{"content":"Deploy to production"}}' > "$TRANSCRIPT"
TRANSCRIPT_JSON="{\"cwd\":\"/Users/test/myproject\",\"transcript_path\":\"$TRANSCRIPT\"}"
if echo "$TRANSCRIPT_JSON" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "from-claude with transcript"
else
    fail "from-claude with transcript" "non-zero exit"
fi

# 5. --from-claude empty stdin
if echo -n "" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "from-claude empty stdin"
else
    fail "from-claude empty stdin" "non-zero exit"
fi

# 6. --from-claude invalid JSON
if echo "not json" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    fail "from-claude invalid json" "should have exited non-zero"
else
    pass "from-claude invalid json rejects"
fi

# 7. No message and no flags
if "$AHOY" 2>/dev/null; then
    fail "no args" "should have exited non-zero"
else
    pass "no args rejects"
fi

# 8. --help
HELP_OUTPUT=$("$AHOY" --help 2>&1 || true)
if echo "$HELP_OUTPUT" | grep -q "Usage"; then
    pass "--help shows usage"
else
    fail "--help" "output does not contain 'Usage'"
fi

# 9. Trailing slash in cwd
TRAILING_JSON='{"cwd":"/foo/bar/"}'
STDERR_OUTPUT=$(echo "$TRAILING_JSON" | "$AHOY" --from-claude -t "Test" 2>&1 >/dev/null || true)
# We can't easily inspect the notification body from outside, but we verify it doesn't crash
if echo "$TRAILING_JSON" | "$AHOY" --from-claude -t "Test" 2>/dev/null; then
    pass "trailing slash cwd doesn't crash"
else
    fail "trailing slash cwd" "non-zero exit"
fi

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
```

- [ ] **Step 2: Make it executable**

Run: `chmod +x tests/integration.sh`

- [ ] **Step 3: Commit**

```bash
git add tests/integration.sh
git commit -m "test: add shell-based integration tests"
```

---

### Task 3: Update build system (Makefile, Info.plist, justfile)

**Files:**
- Modify: `swift/Makefile`
- Modify: `swift/Info.plist:18` (`ahoy-notify` → `ahoy`)
- Modify: `justfile`

- [ ] **Step 1: Update `swift/Info.plist`**

Change line 18 from `ahoy-notify` to `ahoy`:

```xml
    <key>CFBundleExecutable</key>
    <string>ahoy</string>
```

- [ ] **Step 2: Rewrite `swift/Makefile`**

```makefile
# Ahoy — native macOS notification CLI
# Single Swift binary, no package manager

SWIFT_SRC = ahoy.swift
SWIFT_BIN = ahoy
INFO_PLIST = Info.plist
APP_BUNDLE = Ahoy.app
APP_BUNDLE_PATH = ../$(APP_BUNDLE)

INSTALL_DIR = $(HOME)/.ahoy
INSTALL_APP = $(INSTALL_DIR)/$(APP_BUNDLE)
INSTALL_BIN = $(INSTALL_DIR)/bin/$(SWIFT_BIN)

.PHONY: all clean build sign install uninstall test

all: build sign

build:
	@echo "Building $(SWIFT_BIN)..."
	swiftc -O -target arm64-apple-macos13.0 -o $(SWIFT_BIN) $(SWIFT_SRC) \
		-Xlinker -sectcreate -Xlinker __TEXT -Xlinker __info_plist -Xlinker $(INFO_PLIST)
	@echo "Copying to app bundle..."
	mkdir -p $(APP_BUNDLE_PATH)/Contents/MacOS
	mkdir -p $(APP_BUNDLE_PATH)/Contents/Resources
	cp $(SWIFT_BIN) $(APP_BUNDLE_PATH)/Contents/MacOS/
	cp $(INFO_PLIST) $(APP_BUNDLE_PATH)/Contents/Info.plist
	cp -n icons/ahoy-icon-512.png $(APP_BUNDLE_PATH)/Contents/Resources/ 2>/dev/null || true
	cp -n icons/ahoy-icon-128.png $(APP_BUNDLE_PATH)/Contents/Resources/ 2>/dev/null || true
	@echo "Build complete."

sign:
	@echo "Signing app bundle..."
	codesign --force --deep --sign - $(APP_BUNDLE_PATH)
	@echo "Registering with Launch Services..."
	/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f $(APP_BUNDLE_PATH)
	@echo "Signing complete."

install: all
	@echo "Installing to $(INSTALL_DIR)..."
	mkdir -p $(INSTALL_DIR)/bin
	rm -rf $(INSTALL_APP)
	cp -R $(APP_BUNDLE_PATH) $(INSTALL_APP)
	ln -sf $(INSTALL_APP)/Contents/MacOS/$(SWIFT_BIN) $(INSTALL_BIN)
	/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f $(INSTALL_APP)
	@echo "Installed to $(INSTALL_APP)"
	@echo "Symlinked $(INSTALL_BIN) → $(INSTALL_APP)/Contents/MacOS/$(SWIFT_BIN)"

uninstall:
	@echo "Removing $(INSTALL_APP)..."
	rm -rf $(INSTALL_APP)
	rm -f $(INSTALL_BIN)
	@echo "Uninstalled."

test: build
	@echo "Running integration tests..."
	bash ../tests/integration.sh $(APP_BUNDLE_PATH)/Contents/MacOS/$(SWIFT_BIN)

clean:
	rm -f $(SWIFT_BIN)
	rm -rf $(APP_BUNDLE_PATH)/Contents/MacOS/$(SWIFT_BIN)
	@echo "Cleaned."
```

- [ ] **Step 3: Rewrite `justfile`**

```justfile
# List available recipes
default:
    @just --list

# Run all checks (build + test)
check:
    make -C swift build sign
    make -C swift test

# Run tests
test:
    make -C swift test

# Build release binary
build:
    make -C swift build sign

# Install locally
install:
    make -C swift install
```

- [ ] **Step 4: Verify build works**

Run: `make -C swift clean && make -C swift build sign`
Expected: Binary compiles and is code-signed without errors

- [ ] **Step 5: Verify tests pass**

Run: `make -C swift test`
Expected: All 9 integration tests pass

- [ ] **Step 6: Commit**

```bash
git add swift/Makefile swift/Info.plist justfile
git commit -m "build: update Makefile, Info.plist, justfile for Swift-only build"
```

---

### Task 4: Update hooks, install script, and release config

**Files:**
- Modify: `hooks/hooks.json`
- Modify: `install.sh`
- Modify: `release-please-config.json`

- [ ] **Step 1: Update `hooks/hooks.json` — drop `send` from all commands**

Replace the three `command` values. Change `ahoy send` to `ahoy` in each. The full file:

```json
{
  "description": "Ahoy notification hooks for Claude Code",
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "$HOME/.ahoy/bin/ahoy --from-claude -t 'Claude Code' --activate \"$__CFBundleIdentifier\"",
            "timeout": 5000
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "idle_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "$HOME/.ahoy/bin/ahoy -t 'Claude Code' 'Waiting for your input' --activate \"$__CFBundleIdentifier\"",
            "timeout": 5000
          }
        ]
      },
      {
        "matcher": "permission_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "$HOME/.ahoy/bin/ahoy --from-claude -t 'Claude Code' --activate \"$__CFBundleIdentifier\"",
            "timeout": 5000
          }
        ]
      }
    ]
  }
}
```

- [ ] **Step 2: Update `install.sh`**

Replace the binary install section. Instead of copying a separate `ahoy` binary, create a symlink to the one inside `Ahoy.app`. Replace the full file:

```sh
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

    # Install Ahoy.app bundle
    rm -rf "$AHOY_APP"
    cp -R "${tmpdir}/ahoy/Ahoy.app" "$AHOY_APP"

    # Symlink the binary from inside the app bundle
    mkdir -p "$AHOY_BIN"
    ln -sf "$AHOY_APP/Contents/MacOS/ahoy" "$AHOY_BIN/ahoy"

    # macOS post-install: remove quarantine, code sign, register
    xattr -cr "$AHOY_APP" 2>/dev/null || true
    codesign -s - "$AHOY_APP/Contents/MacOS/ahoy" 2>/dev/null || true
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

    echo "To set up Claude Code notifications, run these inside Claude Code:"
    echo ""
    echo "  /plugin marketplace add raiderrobert/ahoy"
    echo "  /plugin install ahoy-hooks@ahoy"
    echo ""
    echo "Test it with: $AHOY_BIN/ahoy 'Hello from Ahoy!'"
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
```

- [ ] **Step 3: Create `version.txt` for release-please**

The `simple` release type reads version from `version.txt` in the repo root. Create it with the current version:

```
0.2.3
```

- [ ] **Step 4: Update `release-please-config.json`**

Change `release-type` from `rust` to `simple`. Keep everything else:

```json
{
  "$schema": "https://raw.githubusercontent.com/googleapis/release-please/main/schemas/config.json",
  "packages": {
    ".": {
      "release-type": "simple",
      "include-component-in-tag": false,
      "bump-minor-pre-major": false,
      "bump-patch-for-minor-pre-major": true,
      "extra-files": [
        {
          "type": "json",
          "path": ".claude-plugin/marketplace.json",
          "jsonpath": "$.version"
        },
        {
          "type": "json",
          "path": ".claude-plugin/plugin.json",
          "jsonpath": "$.version"
        }
      ],
      "changelog-sections": [
        { "type": "feat", "section": "Features" },
        { "type": "fix", "section": "Bug Fixes" },
        { "type": "chore", "section": "Miscellaneous" },
        { "type": "docs", "section": "Documentation" },
        { "type": "refactor", "section": "Code Refactoring" }
      ]
    }
  }
}
```

- [ ] **Step 5: Commit**

```bash
git add hooks/hooks.json install.sh version.txt release-please-config.json
git commit -m "chore: update hooks, installer, and release config for Swift-only binary"
```

---

### Task 5: Update CI/CD workflows

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Rewrite `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-and-test:
    strategy:
      matrix:
        os: [macos-14, macos-15]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: make -C swift build sign

      - name: Run integration tests
        run: make -C swift test
```

- [ ] **Step 2: Rewrite `.github/workflows/release.yml`**

```yaml
name: Release

on:
  workflow_dispatch:
    inputs:
      tag:
        description: "Git tag to build and release (e.g. v0.3.0)"
        required: true
        type: string

permissions:
  contents: write

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.tag }}

      - name: Build Swift binary
        run: make -C swift build sign

      - name: Create release archive
        run: |
          mkdir -p release/ahoy
          cp -R Ahoy.app release/ahoy/
          cp install.sh release/ahoy/
          cp uninstall.sh release/ahoy/
          cp README.md release/ahoy/
          cd release && tar -czvf ahoy-aarch64-macos.tar.gz ahoy

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ahoy-aarch64-macos
          path: release/ahoy-aarch64-macos.tar.gz

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: ahoy-aarch64-macos

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ inputs.tag }}
          files: ahoy-aarch64-macos.tar.gz
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml .github/workflows/release.yml
git commit -m "ci: remove Rust steps, Swift-only build and test"
```

---

### Task 6: Delete Rust artifacts and update misc files

**Files:**
- Delete: `src/` (entire directory)
- Delete: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`
- Delete: `tests/fixtures/`, `tests/CLAUDE.md` (keep `tests/integration.sh`)
- Delete: `swift/ahoy-notify.swift` (replaced by `swift/ahoy.swift`)
- Delete: `lefthook.yml` (references `cargo fmt` which no longer exists)
- Modify: `swift/.gitignore` (ignore `ahoy` instead of `ahoy-notify`)

- [ ] **Step 1: Delete Rust source and config**

```bash
git rm -r src/
git rm Cargo.toml Cargo.lock rust-toolchain.toml
```

- [ ] **Step 2: Delete old test fixtures (preserve `tests/integration.sh`)**

```bash
git rm -r tests/fixtures/
git rm tests/CLAUDE.md
```

- [ ] **Step 3: Delete old Swift source**

```bash
git rm swift/ahoy-notify.swift
```

- [ ] **Step 4: Delete `lefthook.yml`**

The pre-commit hook runs `cargo fmt --check` which will fail without Cargo.toml:

```bash
git rm lefthook.yml
```

- [ ] **Step 5: Update `swift/.gitignore`**

Replace `ahoy-notify` with `ahoy`:

```
# Compiled binary
ahoy

# Code signature
_CodeSignature/
```

- [ ] **Step 6: Verify the build still works after deletion**

Run: `make -C swift clean && make -C swift build sign && make -C swift test`
Expected: Build succeeds, all integration tests pass

- [ ] **Step 7: Commit**

```bash
git add swift/.gitignore
git commit -m "chore: remove Rust codebase, old Swift source, and stale config"
```

---

### Task 7: Final verification

- [ ] **Step 1: Full clean build**

Run: `make -C swift clean && make -C swift build sign`
Expected: Compiles and signs without errors

- [ ] **Step 2: Run all integration tests**

Run: `make -C swift test`
Expected: All 9 tests pass

- [ ] **Step 3: Manual smoke test — direct message**

Run: `Ahoy.app/Contents/MacOS/ahoy "Hello from Swift rewrite" -t "Ahoy"`
Expected: Notification appears

- [ ] **Step 4: Manual smoke test — from-claude with tool**

Run: `echo '{"cwd":"/tmp/myproject","tool_name":"Bash","tool_input":{"command":"npm test"}}' | Ahoy.app/Contents/MacOS/ahoy --from-claude -t "Claude Code"`
Expected: Notification with body `[myproject] Bash: npm test`

- [ ] **Step 5: Manual smoke test — help**

Run: `Ahoy.app/Contents/MacOS/ahoy --help`
Expected: Usage text printed to stderr

- [ ] **Step 6: Verify `just check` works**

Run: `just check`
Expected: Build + tests pass

- [ ] **Step 7: Verify no stale files remain**

Run: `ls src/ 2>&1` and `ls Cargo.toml 2>&1`
Expected: Both should say "No such file or directory"
