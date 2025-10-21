// Copyright 2025. This file is part of TFRAlert.

// TFRAlert is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

// TFRAlert is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along with TFRAlert. If not, see <https://www.gnu.org/licenses/>.

use crate::logic::ParsedTFREvent;
use notify_rust::Notification;

pub struct NotificationText {
    title: String,
    body: String,
}

fn get_notification_text(events: &[ParsedTFREvent]) -> Option<NotificationText> {
    match events.len() {
        0 => None,
        1 => build_single_notification(&events[0]),
        _ => build_batch_notification(events),
    }
}

fn build_single_notification(event: &ParsedTFREvent) -> Option<NotificationText> {
    let mut title = format!("New TFR: {}", event.notam_id);
    let mut body = String::new();

    if !event.location.is_empty() {
        title.push_str(&format!(" ({})", event.location));
    }
    if !event.reason.is_empty() {
        body.push_str(&format!("Reason: {}\n", event.reason));
    }
    // if !event.airspace.altitude.is_empty() {
    //     body.push_str(&format!("Altitude: {}\n", event.airspace.altitude));
    // }
    // if !event.airspace.center.is_empty() {
    //     body.push_str(&format!("Center: {}\n", event.airspace.center));
    // }
    if !event.restrictions.is_empty() {
        body.push_str(&format!("Restrictions: {}\n", event.restrictions));
    }

    Some(NotificationText {
        title: title,
        body: body,
    })
}

fn build_batch_notification(events: &[ParsedTFREvent]) -> Option<NotificationText> {
    if events.is_empty() {
        return None;
    }
    let mut body = String::new();
    for e in events {
        let loc = if e.location.trim().is_empty() {
            "(Unknown)".to_string()
        } else {
            e.location.clone()
        };

        // Build the line of text
        body.push_str(&format!("* {}: {}\n", loc, e.reason));
    }

    Some(NotificationText {
        title: format!("{} TFRs", events.len()),
        body,
    })
}

pub fn notify(events: &Vec<ParsedTFREvent>) {
    if let Some(notif) = get_notification_text(events) {
        #[cfg(target_os = "linux")]
        show_notification(&notif.title, &notif.body);

        #[cfg(target_os = "windows")]
        show_notification(&notif.title, &notif.body);

        #[cfg(target_os = "macos")]
        show_notification(&notif.title, &notif.body);
    }
}

#[cfg(target_os = "linux")]
fn show_notification(title: &str, body: &str) {
    use notify_rust::Notification;
    let _ = Notification::new().summary(title).body(body).show();
}

#[cfg(target_os = "macos")]
fn show_notification(title: &str, body: &str) {
    use std::process::Command;
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            r#"display notification "{}" with title "{}""#,
            body.replace('"', "\\\""),
            title.replace('"', "\\\"")
        ))
        .spawn();
}

// TODO
// #[cfg(target_os = "windows")]
// fn show_notification(title: &str, body: &str) {
//     use toast::Toast;
//     let mut toast = Toast::new(Toast::TODO_MAKE_POWERSHELL_APP_ID);
//     toast.title(title);
//     toast.text1(body);
//     let _ = toast.show();
// }
