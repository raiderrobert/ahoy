use anyhow::Result;
use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::{NSError, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotificationRequest,
    UNNotificationSound, UNUserNotificationCenter,
};
use std::sync::mpsc;
use tracing::info;

use crate::client::message::Notification;

pub fn show(notification: &Notification) -> Result<()> {
    info!("Attempting to show macOS notification via UNUserNotificationCenter...");

    // Get the current notification center
    let center = unsafe { UNUserNotificationCenter::currentNotificationCenter() };

    // Request authorization (needed on first run)
    request_authorization(&center)?;

    // Create notification content
    let content = unsafe {
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(&notification.title));
        content.setBody(&NSString::from_str(&notification.body));
        content.setSound(Some(&UNNotificationSound::defaultSound()));
        content
    };

    // Create a unique identifier for this notification
    let identifier = NSString::from_str(&format!("ahoy-{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));

    // Create the notification request
    let request = unsafe {
        UNNotificationRequest::requestWithIdentifier_content_trigger(&identifier, &content, None)
    };

    // Send the notification
    let (tx, rx) = mpsc::channel();
    let block = RcBlock::new(move |error: *mut NSError| {
        if error.is_null() {
            let _ = tx.send(Ok(()));
        } else {
            let err_msg = unsafe {
                let err = &*error;
                err.localizedDescription().to_string()
            };
            let _ = tx.send(Err(err_msg));
        }
    });

    unsafe {
        center.addNotificationRequest_withCompletionHandler(&request, Some(&block));
    }

    // Wait for the result
    match rx.recv() {
        Ok(Ok(())) => {
            info!("Notification shown successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            info!("Notification error: {}", e);
            anyhow::bail!("Failed to show notification: {}", e)
        }
        Err(e) => {
            anyhow::bail!("Failed to receive notification result: {}", e)
        }
    }
}

fn request_authorization(center: &Retained<UNUserNotificationCenter>) -> Result<()> {
    let options = UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound | UNAuthorizationOptions::Badge;

    let (tx, rx) = mpsc::channel();
    let block = RcBlock::new(move |granted: bool, error: *mut NSError| {
        if !error.is_null() {
            let err_msg = unsafe {
                let err = &*error;
                err.localizedDescription().to_string()
            };
            let _ = tx.send(Err(err_msg));
        } else {
            let _ = tx.send(Ok(granted));
        }
    });

    unsafe {
        center.requestAuthorizationWithOptions_completionHandler(options, &block);
    }

    match rx.recv() {
        Ok(Ok(granted)) => {
            if granted {
                info!("Notification authorization granted");
            } else {
                info!("Notification authorization denied");
            }
            Ok(())
        }
        Ok(Err(e)) => {
            anyhow::bail!("Authorization error: {}", e)
        }
        Err(e) => {
            anyhow::bail!("Failed to receive authorization result: {}", e)
        }
    }
}
