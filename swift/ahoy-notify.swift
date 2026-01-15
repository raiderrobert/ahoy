#!/usr/bin/env swift

import Foundation
import AppKit
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
    } else {
        i += 1
    }
}

// Set up the delegate with the activation bundle ID
notificationDelegate.activateBundleId = activateBundleId
NSUserNotificationCenter.default.delegate = notificationDelegate

// MARK: - Focus Check
// If the source terminal is focused, user is already watching - don't notify.
if let bundleId = activateBundleId {
    let frontmostApp = NSWorkspace.shared.frontmostApplication
    let frontmostBundleId = frontmostApp?.bundleIdentifier

    if frontmostBundleId == bundleId {
        fputs("Terminal is focused (\(bundleId)), skipping notification\n", stderr)
        exit(0)
    }
    fputs("Terminal not focused (front: \(frontmostBundleId ?? "nil")), showing notification\n", stderr)
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
fputs("Notification delivered\n", stderr)

// If we have an activation target, wait for user to click with retry logic:
// - Wait 30s for response
// - If no response, retry once
// - Wait another 30s
// - Give up
if activateBundleId != nil {
    let retryDelay: TimeInterval = 30

    // First wait
    var timeout = Date(timeIntervalSinceNow: retryDelay)
    while !notificationDelegate.didActivate && Date() < timeout {
        RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.1))
    }

    // If user didn't respond, retry once
    if !notificationDelegate.didActivate {
        fputs("No response after 30s, retrying notification\n", stderr)

        // Remove old notification and deliver again
        NSUserNotificationCenter.default.removeDeliveredNotification(notification)
        NSUserNotificationCenter.default.deliver(notification)

        // Second wait
        timeout = Date(timeIntervalSinceNow: retryDelay)
        while !notificationDelegate.didActivate && Date() < timeout {
            RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.1))
        }

        if !notificationDelegate.didActivate {
            fputs("No response after retry, giving up\n", stderr)
        }
    }
} else {
    RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.5))
}
