// Copyright 2025. This file is part of TFRAlert.
// Licensed under GPLv3 or later.

use async_std::task::sleep;
use dioxus::prelude::*;

mod logic;
mod notify;

fn main() {
    dioxus::launch(app);
}

const NOTAM_DETAIL_URL_PRETTY: &str = "https://tfr.faa.gov/tfr3/?page=detail_";
const MATCHES: &str = "tfr_matches.json";
const REFRESH_SECONDS: u64 = 600; // configurable later

#[derive(Debug, Clone)]
enum LoadState {
    Loading,
    Loaded(FeedResult),
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct FeedResult {
    events: Vec<logic::ParsedTFREvent>,
    unseen_count: usize,
    today_count: usize,
    city_today_count: usize,
}

#[component]
pub fn app() -> Element {
    // refresh_counter is incremented to trigger new fetches
    let mut refresh_counter = use_signal(|| 0u64);
    let mut feed_state = use_signal(|| LoadState::Loading);

    use_future({
        let mut feed_state = feed_state.clone();
        let refresh_counter = refresh_counter();
        move || async move {
            feed_state.set(LoadState::Loading);
            match logic::refresh_tfr_results().await {
                Ok(result) => feed_state.set(LoadState::Loaded(result)),
                Err(e) => feed_state.set(LoadState::Error(e.to_string())),
            }
        }
    });

    use_future({
        let mut refresh_counter = refresh_counter.clone();
        move || async move {
            loop {
                sleep(std::time::Duration::from_secs(REFRESH_SECONDS)).await;
                refresh_counter += 1;
            }
        }
    });

    let content = match feed_state() {
        LoadState::Loading => rsx!(p { "Loading feed..." }),
        LoadState::Error(e) => rsx!(p { "Error loading feed: {e}" }),
        LoadState::Loaded(result) => {
            if result.unseen_count > 0 {
                let new_events = result
                    .events
                    .iter()
                    .take(result.unseen_count)
                    .rev() // chronological order for notifications
                    .cloned()
                    .collect::<Vec<_>>();
                std::thread::spawn(move || notify::notify(&new_events));
            }

            let event_items = result
                .events
                .iter()
                .map(|event| {
                    let notam_id = &event.notam_id;
                    let city = &event.location;
                    let date = &event.issue_date;
                    let url = format!(
                        "{}{}",
                        NOTAM_DETAIL_URL_PRETTY,
                        &event.notam_id.replace("/", "_")
                    );

                    rsx! {
                        li { class: "event-item",
                            div {
                                a {
                                    class: "notam-link",
                                    href: "{url}",
                                    target: "_blank",
                                    "{notam_id}"
                                }
                                span { "{date} {city}" }
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();

            let summary = format!(
                "Showing {} items (type: Security, altitude: 0–400 ft AGL)",
                event_items.len()
            );

            rsx! {
                div { class: "app-container",
                    h2 { "TFRAlert" }

                    div { class: "header-row",
                        p { class: "summary", "{summary}" }

                        button {
                            class: "refresh-button",
                            onclick: move |_| refresh_counter += 1,
                            "Refresh"
                        }
                    }

                    ul { class: "event-list", {event_items.into_iter()} }

                    p { style: "margin-top: 1em; font-style: italic;",
                        "For details of all events see {MATCHES}"
                    }
                }
            }
        }
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/style.css") }
        {content}
    }
}
