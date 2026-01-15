#!/usr/bin/env swift

import Foundation
import AppKit
import CoreGraphics
import ObjectiveC

// MARK: - Bundle Identifier Swizzling (like terminal-notifier)
// This makes macOS think notifications come from our app, showing our icon on the left

let fakeBundleIdentifier = "rs.ahoy.notify.fresh"

// Use block-based implementation replacement instead of method swizzling
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
    fputs("Bundle swizzling installed for: \(fakeBundleIdentifier)\n", stderr)
}

// Install the hook before anything else
installFakeBundleIdentifierHook()

// Verify swizzling works
fputs("Bundle.main.bundleIdentifier = \(Bundle.main.bundleIdentifier ?? "nil")\n", stderr)

// MARK: - Notification Delegate for handling clicks
class NotificationDelegate: NSObject, NSUserNotificationCenterDelegate {
    var activateBundleId: String?
    var didActivate = false

    func userNotificationCenter(_ center: NSUserNotificationCenter, didActivate notification: NSUserNotification) {
        didActivate = true
        if let bundleId = activateBundleId {
            // Find and activate the running app
            let runningApps = NSWorkspace.shared.runningApplications.filter { $0.bundleIdentifier == bundleId }
            if let app = runningApps.first {
                // Activate existing instance - use activate() without deprecated options
                app.activate()
            } else {
                // App not running - use open command which is more reliable
                let task = Process()
                task.launchPath = "/usr/bin/open"
                task.arguments = ["-b", bundleId]
                try? task.run()
            }
        }
        // Exit after handling
        exit(0)
    }

    func userNotificationCenter(_ center: NSUserNotificationCenter, shouldPresent notification: NSUserNotification) -> Bool {
        return true // Always show, even if app is frontmost
    }
}

let notificationDelegate = NotificationDelegate()

// Initialize NSApplication so macOS recognizes us as a proper app
let app = NSApplication.shared
app.setActivationPolicy(.accessory)

// Parse command line arguments
let args = CommandLine.arguments
guard args.count >= 3 else {
    fputs("Usage: ahoy-notify <title> <body> [--sound <name>] [--activate <bundle-id>]\n", stderr)
    exit(1)
}

let title = args[1]
let body = args[2]

var soundName = "Glass"
var iconPath: String? = nil
var activateBundleId: String? = nil
var idleThreshold: Double = 30.0  // Suppress notification if user active within this many seconds

// Default icon path - check Resources directory (for app bundle) then same directory as binary
// Prefer 512px icon for Retina displays, fallback to 128px
let binaryPath = URL(fileURLWithPath: args[0]).deletingLastPathComponent()
let resourcesDir = binaryPath.deletingLastPathComponent().appendingPathComponent("Resources")
let resources512 = resourcesDir.appendingPathComponent("ahoy-icon-512.png").path
let resources128 = resourcesDir.appendingPathComponent("ahoy-icon-128.png").path
let sameDir512 = binaryPath.appendingPathComponent("ahoy-icon-512.png").path
let sameDir128 = binaryPath.appendingPathComponent("ahoy-icon-128.png").path

if FileManager.default.fileExists(atPath: resources512) {
    iconPath = resources512
} else if FileManager.default.fileExists(atPath: resources128) {
    iconPath = resources128
} else if FileManager.default.fileExists(atPath: sameDir512) {
    iconPath = sameDir512
} else if FileManager.default.fileExists(atPath: sameDir128) {
    iconPath = sameDir128
}

var i = 3
while i < args.count {
    if args[i] == "--sound" && i + 1 < args.count {
        soundName = args[i + 1]
        i += 2
    } else if args[i] == "--icon" && i + 1 < args.count {
        iconPath = args[i + 1]
        i += 2
    } else if args[i] == "--activate" && i + 1 < args.count {
        activateBundleId = args[i + 1]
        i += 2
    } else if args[i] == "--idle-threshold" && i + 1 < args.count {
        idleThreshold = Double(args[i + 1]) ?? 30.0
        i += 2
    } else if args[i] == "--force" {
        idleThreshold = 0  // Disable idle check
        i += 1
    } else {
        i += 1
    }
}

// Set up the delegate with the activation bundle ID
notificationDelegate.activateBundleId = activateBundleId
NSUserNotificationCenter.default.delegate = notificationDelegate

// MARK: - Idle Detection
// Only suppress notifications if the source app (terminal) is frontmost AND user is active.
// If user switched to another app, always notify - they're not watching Claude anymore.
if idleThreshold > 0 {
    // Check if the source app (terminal) is the frontmost app
    let frontmostApp = NSWorkspace.shared.frontmostApplication
    let frontmostBundleId = frontmostApp?.bundleIdentifier

    // Only apply idle suppression if the terminal is still in focus
    let terminalIsFront = activateBundleId != nil && frontmostBundleId == activateBundleId

    if terminalIsFront {
        // Get seconds since last keyboard or mouse event
        let keyboardIdle = CGEventSource.secondsSinceLastEventType(.hidSystemState, eventType: .keyDown)
        let mouseIdle = CGEventSource.secondsSinceLastEventType(.hidSystemState, eventType: .mouseMoved)
        let clickIdle = CGEventSource.secondsSinceLastEventType(.hidSystemState, eventType: .leftMouseDown)

        // User is considered active if any input within threshold
        let minIdle = min(keyboardIdle, mouseIdle, clickIdle)

        if minIdle < idleThreshold {
            fputs("Terminal focused + user active (idle: \(String(format: "%.1f", minIdle))s < \(String(format: "%.0f", idleThreshold))s), skipping notification\n", stderr)
            exit(0)
        }
        fputs("Terminal focused but user idle for \(String(format: "%.1f", minIdle))s, showing notification\n", stderr)
    } else {
        fputs("Terminal not focused (front: \(frontmostBundleId ?? "nil")), showing notification\n", stderr)
    }
}

// Use deprecated but reliable NSUserNotification
// This doesn't require explicit authorization like UNUserNotificationCenter
let notification = NSUserNotification()
notification.title = title
notification.informativeText = body
notification.soundName = soundName

// The left side now shows the app icon via bundle swizzling
// No need to set contentImage (right side) anymore

// Deliver the notification
NSUserNotificationCenter.default.deliver(notification)

// If we have an activation target, wait for user to click (up to 30 seconds)
// Otherwise just give it a moment to deliver
if activateBundleId != nil {
    let timeout = Date(timeIntervalSinceNow: 30)
    while !notificationDelegate.didActivate && Date() < timeout {
        RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.1))
    }
} else {
    RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.5))
}
