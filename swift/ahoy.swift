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
