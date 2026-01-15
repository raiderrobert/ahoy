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

// Initialize NSApplication so macOS recognizes us as a proper app
let app = NSApplication.shared
app.setActivationPolicy(.accessory)

// Parse command line arguments
let args = CommandLine.arguments
guard args.count >= 3 else {
    fputs("Usage: ahoy-notify <title> <body> [--sound <name>] [--icon <path>]\n", stderr)
    exit(1)
}

let title = args[1]
let body = args[2]

var soundName = "Glass"
var iconPath: String? = nil

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
    } else {
        i += 1
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

// Give it a moment to deliver
RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.5))
