use dioxus::prelude::*;
use notify_rust::Notification;
use tokio::time::{Duration, interval};

mod logic;

fn main() {
    dioxus::launch(app);
}

#[derive(Debug, Clone, Default)]
struct FeedResult {
    events: Vec<logic::TFREvent>,
    unseen_count: usize,
    today_count: usize,
    city_today_count: usize,
}

const MATCHES: &str = "tfr_matches.json";
const REFRESH_SECONDS: u64 = 600;

#[component]
pub fn app() -> Element {
    let mut feedresult = use_action(|| async {
        let mut tfr_matches = logic::load_matched_cache_sorted();
        let cached_match_count = tfr_matches.len();

        // Check for new matches
        if let Ok(new_matches) = logic::check_feed().await {
            if !new_matches.is_empty() {
                for e in new_matches.iter() {
                    // TODO: some NOTAMs refer to or update the ID of another NOTAM
                    if !tfr_matches.iter().any(|m| m.notam_id == e.notam_id) {
                        tfr_matches.insert(0, e.clone());
                    }
                }

                // Notify for new matches (TODO)
                for entry in &new_matches {
                    #[cfg(not(target_os = "macos"))]
                    let _ = Notification::new()
                        .summary("New Feed Match")
                        .body(&format!("NOTAM {}", entry.notam_id))
                        .show();
                }
            }
        }

        let (today_total, city_count) = logic::summarize_matched_events(&tfr_matches);
        let new_since_last = today_total - cached_match_count;

        dioxus::Ok(FeedResult {
            events: tfr_matches,
            unseen_count: new_since_last,
            today_count: today_total,
            city_today_count: city_count,
        })
    });
    // Background refresh every REFRESH_INTERVAL_SECS
    use_effect({
        let mut feedresult = feedresult.clone();
        move || {
            spawn(async move {
                let mut ticker = interval(Duration::from_secs(REFRESH_SECONDS));
                loop {
                    ticker.tick().await;
                    feedresult.call();
                }
            });
        }
    });

    let Some(Ok(signal)) = feedresult.value() else {
        return rsx! { p { "Loading feed..." } };
    };

    let result: FeedResult = signal();

    let event_word = if result.unseen_count == 1 {
        "event"
    } else {
        "events"
    };
    let city_word = if result.city_today_count == 1 {
        "city"
    } else {
        "cities"
    };

    let event_items = result
        .events
        .iter()
        .map(|event| {
            let notam_id = &event.notam_id;
            let city = event
                .parsed
                .as_ref()
                .map(|p| p.location.clone())
                .unwrap_or_else(|| "(Unknown)".to_string());

            rsx! {
                li { class: "event-item",
                    strong { "{notam_id}" }
                    span { " — {city}" }
                }
            }
            .unwrap()
        })
        .collect::<Vec<_>>();

    rsx! {
        div { class: "app-container",
            h1 { "TFRAlert" }

            p { class: "summary",
                "{result.unseen_count} new {event_word}. Today {result.today_count} total in {result.city_today_count} {city_word}."
            }

            button {
                class: "refresh-button",
                onclick: move |_| feedresult.call(),
                "Refresh"
            }

            ul { class: "event-list",
                {event_items.into_iter()}
            }

            p { style: "margin-top: 1em; font-style: italic;",
                "For full list of events see {MATCHES}"
            }
        }
    }
}
