// Copyright 2025. This file is part of TFRAlert.

// TFRAlert is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

// TFRAlert is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along with TFRAlert. If not, see <https://www.gnu.org/licenses/>.

use crate::tfr_core::ParsedTFREvent;
use notify_rust::Notification;

#[derive(Debug)]
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

pub fn notify(events: &[ParsedTFREvent]) {
    if let Some(notif) = get_notification_text(events) {
        show_notification(&notif.title, &notif.body);
    } else {
        log::debug!("No new TFRs");
    }
}

fn build_single_notification(event: &ParsedTFREvent) -> Option<NotificationText> {
    let mut title = format!("New TFR: {}", event.notam_id);
    if !event.location.is_empty() {
        title.push_str(&format!(" ({})", event.location));
    }

    let mut body = String::new();
    if !event.reason.is_empty() {
        body.push_str(&format!("Reason: {}\n", event.reason));
    }
    if !event.restrictions.is_empty() {
        body.push_str(&format!("Restrictions: {}\n", event.restrictions));
    }
    if !event.begin.is_empty() && !event.end.is_empty() {
        body.push_str(&format!("{} - {}\n", event.begin, event.end));
    }

    Some(NotificationText { title, body })
}

fn build_batch_notification(events: &[ParsedTFREvent]) -> Option<NotificationText> {
    if events.is_empty() {
        return None;
    }

    // collect a few sample locations
    let mut cities: Vec<String> = events
        .iter()
        .filter_map(|e| {
            if e.location.trim().is_empty() {
                None
            } else {
                Some(e.location.clone())
            }
        })
        .collect();

    cities.sort();
    cities.dedup();

    let preview = if cities.len() <= 3 {
        cities.join(", ")
    } else {
        format!(
            "{}, …",
            cities
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    Some(NotificationText {
        title: format!(
            "{} new TFR{}",
            events.len(),
            if events.len() > 1 { "s" } else { "" }
        ),
        body: format!("Locations: {preview}"),
    })
}

fn show_notification(title: &str, body: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(body)
            .show();
    }

    #[cfg(target_os = "macos")]
    {
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

    #[cfg(target_os = "windows")]
    {
        use winrt_notification::{Duration, Sound, Toast};
        let _ = Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(body)
            .sound(Some(Sound::Default))
            .duration(Duration::Short)
            .show();
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Browser fallback
        web_sys::console::log_2(&title.into(), &body.into());
        if let Some(window) = web_sys::window() {
            let _ = window.alert_with_message(&format!("{title}\n{body}"));
        }
    }
}
