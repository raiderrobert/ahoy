#!/usr/bin/env swift

import Foundation
import AppKit

// Parse command line arguments
let args = CommandLine.arguments
guard args.count >= 3 else {
    fputs("Usage: ahoy-notify <title> <body> [--sound <name>]\n", stderr)
    exit(1)
}

let title = args[1]
let body = args[2]

var soundName = "Glass"

var i = 3
while i < args.count {
    if args[i] == "--sound" && i + 1 < args.count {
        soundName = args[i + 1]
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

// Deliver the notification
NSUserNotificationCenter.default.deliver(notification)

// Give it a moment to deliver
RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.5))
