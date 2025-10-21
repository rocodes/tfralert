// Copyright 2025. This file is part of TFRAlert.

// TFRAlert is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

// TFRAlert is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along with TFRAlert. If not, see <https://www.gnu.org/licenses/>.

use dioxus::prelude::*;

mod logic;
mod notify;

fn main() {
    dioxus::launch(app);
}

#[derive(Debug, Clone, Default)]
struct FeedResult {
    events: Vec<logic::ParsedTFREvent>,
    unseen_count: usize,
    today_count: usize,
    city_today_count: usize,
}

// not for parsing
const NOTAM_DETAIL_URL_PRETTY: &str = "https://tfr.faa.gov/tfr3/?page=detail_";
const MATCHES: &str = "tfr_matches.json";
const REFRESH_SECONDS: u64 = 600; // todo configurable

#[component]
pub fn app() -> Element {
    let mut feedresult = use_action(|| async {
        logic::refresh_tfr_results()
            .await
            .map_err(dioxus::Error::from)
            .into()
    });

    use_effect({
        let mut feedresult = feedresult.clone();
        move || {
            feedresult.call();
        }
    });

    let Some(Ok(signal)) = feedresult.value() else {
        return rsx! { p { "Loading feed..." } };
    };

    let result: FeedResult = signal();

    // FIXME
    // let event_word = if result.unseen_count == 1 {
    //     "event"
    // } else {
    //     "events"
    // };
    // let city_word = if result.city_today_count == 1 {
    //     "city"
    // } else {
    //     "cities"
    // };

    let result_for_notif = result.clone();

    // TODO
    tokio::task::spawn_blocking(move || {
        notify::notify(&result_for_notif.events);
    });

    let event_items = result
        .events
        .iter()
        .map(|event| {
            // TODO
            //let mut expanded = use_signal(|| false);

            let notam_id = &event.notam_id;
            let city = &event.location;
            let date = &event.issue_date;
            // let reason = &event.reason;
            // let restrictions = &event.restrictions;

            let url = format!(
                "{}{}",
                NOTAM_DETAIL_URL_PRETTY,
                &event.notam_id.replace("/", "_")
            );

            rsx! {
                    li { class: "event-item",
                    // TODO expansion on click
                    // onclick: move |_| expanded.set(!expanded()),

                    div {
                        a {
                            class: "notam-link",
                            href: "{url}",
                            target: "_blank",
                            "{notam_id}" }
                        span { "{date} {city}" }
                    }
                    // TODO: these aren't always filled out, show only if more info
                    // if expanded() {
                    //     div { class: "event-details",
                    //         p { "Reason: {reason}" }
                    //         p { "Restrictions: {restrictions}" }
                    //     }
                    // }
                }
            }
            .unwrap()
        })
        .collect::<Vec<_>>();

    // TODO
    // loop {
    //         sleep(tokio::time::Duration::from_secs(REFRESH_SECONDS));
    //         feedresult.call();
    //     }

    rsx! {
        document::Stylesheet { href: asset!("/assets/style.css") }
        div { class: "app-container",
            h2 { "TFRAlert" }

            div { class: "header-row",

            // TODO: this will show since the user has kept the feed
            // todo: need to save/export
            p { class: "summary",
                "Showing {event_items.len()} items (type: Security, altitude: 0-400 ft AGL)"
            }

            button {
                class: "refresh-button",
                onclick: move |_| feedresult.call(),
                "Refresh"
            }
        }

            ul { class: "event-list",
            // items display oldest to newest unless we reverse
            // todo: better date and location parsing to group items
                {event_items.into_iter()}
            }

            p { style: "margin-top: 1em; font-style: italic;",
                "For details of all events see {MATCHES}"
            }
        }
    }
}
