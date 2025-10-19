use dioxus::prelude::*;
use tokio::time::{interval, Duration};
use notify_rust::Notification;

mod logic;

fn main() {
    dioxus::launch(app);
}

struct FeedResult<T: 'static + ?Sized> {
    events: Vec<Event<T>>,
    new_since_last: usize,
    day_total: usize,
    city_day_total: usize,
}

const MATCHED_FILE: &str = "tfr_matches.json";
const REFRESH_INTERVAL_SECS: u64 = 60;

pub fn app() -> Element {
    let mut feedresult = use_action(|| async {
        let mut matches = logic::load_matched_cache_sorted();
        let cached_match_count = matches.len();

        // Check for new matches
        if let Ok(new_matches) = logic::check_feed().await {
            if !new_matches.is_empty() {
                for e in new_matches.iter() {
                    if !matches.iter().any(|m| m.notam_id == e.notam_id) {
                        matches.insert(0, e.clone());
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

        let (today_total, city_count) = logic::summarize_matched_events(&matches);
        let new_since_last = today_total - cached_match_count;
        
        dioxus::Ok(FeedResult{events : matches, new_since_last: new_since_last, day_total: today_total, city_day_total: city_count})
    });
    // Background refresh every REFRESH_INTERVAL_SECS
    use_effect({
        let feedresult = feedresult.clone();
        move || {
            spawn(async move {
                let mut ticker = interval(Duration::from_secs(REFRESH_INTERVAL_SECS));
                loop {
                    ticker.tick().await;
                    feedresult.dispatch(());
                }
            });
        }
    });

    // Await async result
    let Some((matches, new_today, today_total, city_count)) = feedresult.await() else {
        return rsx! { p { "Loading feed..." } };
    };

    let event_word = if *new_today == 1 { "event" } else { "events" };
    let matched_word = if *today_total == 1 { "matched event" } else { "matched events" };
    let city_word = if *city_count == 1 { "city" } else { "cities" };

    rsx! {
        div { class: "app-container",
            h1 { "TFR Alert" }

            p { class: "summary",
                "{new_today} new {event_word}. {today_total} {matched_word} today in {city_count} {city_word}."
            }

            button {
                class: "refresh-button",
                onclick: move |_| feedresult.dispatch(()),
                "Refresh"
            }

            ul { class: "event-list",
                for event in matches.iter() {
                    let notam_id = &event.notam_id;
                    let city = event.parsed
                        .as_ref()
                        .map(|p| p.location.clone())
                        .unwrap_or_else(|| "(Unknown)".to_string());
                    rsx! {
                        li { class: "event-item",
                            strong { "{notam_id}" }
                            span { " — {city}" }
                        }
                    }
                }
            }

            p { style: "margin-top: 1em; font-style: italic;",
                "For full list of events see {MATCHED_FILE}"
            }
        }
    }
}
