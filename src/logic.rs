use anyhow::Result;
use log::{debug, error, info};
use notify_rust::Notification;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};
use tokio::time::Duration;

const RAW_EVENT_CACHE: &str = "tfr_cache.json";
const MATCHED_EVENT_CACHE: &str = "tfr_matches.json";

const JSON_FEED_TFR_URL: &str = "https://tfr.faa.gov/tfrapi/exportTfrList";
const NOTAM_DETAIL_URL: &str = "https://tfr.faa.gov/tfrapi/getWebText?notamId={}";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TFREvent {
    pub notam_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub parsed: Option<TFREventDetail>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TFREventDetail {
    pub notam_id: Option<String>,
    pub issue_date: String,
    pub location: String,
    pub begin: String,
    pub end: String,
    pub reason: String,
    pub r#type: String,
    pub replaced: String,
    pub airspace: Airspace,
    pub restrictions: String,
    pub other_info: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airspace {
    pub center: String,
    pub radius: String,
    pub altitude: String,
    pub effective: Vec<String>,
}

pub async fn download_json_feed(client: &Client) -> Result<Vec<TFREvent>> {
    let resp = client.get(JSON_FEED_TFR_URL).send().await?;
    let json = resp.json::<Vec<TFREvent>>().await?;
    Ok(json)
}

pub fn load_raw_cache() -> Vec<TFREvent> {
    read_cache_file(RAW_EVENT_CACHE)
}

pub fn save_raw_cache(data: &[TFREvent]) -> Result<()> {
    write_cache_file(RAW_EVENT_CACHE, data)
}

pub fn load_matched_cache() -> Vec<TFREvent> {
    read_cache_file(MATCHED_EVENT_CACHE)
}

pub fn save_matched_cache(data: &[TFREvent]) -> Result<()> {
    write_cache_file(MATCHED_EVENT_CACHE, data)
}

fn read_cache_file(path: &str) -> Vec<TFREvent> {
    let path = PathBuf::from(path);
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                error!("Failed to parse JSON from {}: {}", path.display(), e);
                Vec::new()
            }),
            Err(e) => {
                error!("Failed to read cache file {}: {}", path.display(), e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    }
}

fn write_cache_file(path: &str, data: &[TFREvent]) -> Result<()> {
    let serialized = serde_json::to_string_pretty(data)?;
    fs::write(path, serialized)?;
    Ok(())
}

/// event type "SECURITY"
pub fn get_filtered_events(data: &[TFREvent]) -> Vec<TFREvent> {
    data.iter()
        .filter(|e| {
            e.r#type
                .as_deref()
                .unwrap_or("")
                .eq_ignore_ascii_case("SECURITY")
        })
        .cloned()
        .collect()
}

/// events since we last checked, indexed by notam_id
pub fn get_new_events(current: &[TFREvent], cached: &[TFREvent]) -> Vec<TFREvent> {
    let cached_ids: HashSet<_> = cached.iter().map(|e| &e.notam_id).collect();
    current
        .iter()
        .filter(|e| !cached_ids.contains(&e.notam_id))
        .cloned()
        .collect()
}

/// NOTAM detail page
pub async fn fetch_detail_page(client: &Client, notam_id: &str) -> Result<String> {
    let url = NOTAM_DETAIL_URL.replace("{}", notam_id);
    let resp = client.get(&url).send().await?.text().await?;
    Ok(resp)
}

/// keyword search from text file (TODO)
pub fn load_keywords(path: Option<&str>) -> Vec<String> {
    if let Some(path) = path {
        let p = PathBuf::from(path);
        if p.exists() {
            if let Ok(content) = fs::read_to_string(p) {
                return content
                    .lines()
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

// pub fn event_matches_criteria(html_text: &str, keywords: &[String]) -> bool {
//     if keywords.is_empty() {
//         return true;
//     }
//     let lower = html_text.to_lowercase();
//     keywords.iter().any(|kw| lower.contains(kw))
// }

/// Sends desktop notification for a single event.
pub fn notify_event(event: &TFREvent) {
    let mut title = format!("New TFR: {}", event.notam_id);

    if let Some(parsed) = &event.parsed {
        if !parsed.location.is_empty() {
            title.push_str(&format!(" ({})", parsed.location));
        }
        let mut body = String::new();
        if !parsed.reason.is_empty() {
            body.push_str(&format!("Reason: {}\n", parsed.reason));
        }
        if !parsed.airspace.altitude.is_empty() {
            body.push_str(&format!("Altitude: {}\n", parsed.airspace.altitude));
        }
        if !parsed.airspace.center.is_empty() {
            body.push_str(&format!("Center: {}\n", parsed.airspace.center));
        }
        if !parsed.restrictions.is_empty() {
            body.push_str(&format!("Restrictions: {}\n", parsed.restrictions));
        }

        let _ = Notification::new()
            .summary(&title)
            .body(&body)
            .timeout(10_000)
            .show();
    } else {
        let _ = Notification::new()
            .summary(&title)
            .body(&event.description)
            .timeout(10_000)
            .show();
    }
}

/// Desktop notification for multiple events
pub fn notify_batch_event(events: &[TFREvent]) {
    let mut body = String::new();
    for e in events {
        if let Some(parsed) = &e.parsed {
            let loc = if parsed.location.is_empty() {
                "(Unknown)".to_string()
            } else {
                parsed.location.clone()
            };
            body.push_str(&format!("* {}: {}\n", loc, parsed.reason));
        } else {
            body.push_str(&format!(
                "* {}: {}\n",
                e.location.clone().unwrap_or_default(),
                e.description
            ));
        }
    }

    let title = format!("{} new TFRs detected", events.len());
    let _ = Notification::new()
        .summary(&title)
        .body(&body)
        .timeout(10_000)
        .show();
}

async fn process_feed(keywords: &[String], notify: bool) -> Result<Vec<TFREvent>> {
    use log::{debug, error, info};

    let client = Client::new();

    debug!("Check feed");
    let current_data = download_json_feed(&client).await?;
    info!("Downloaded {} total items", current_data.len());

    let cached_data = load_raw_cache();
    let current = get_filtered_events(&current_data);
    let cached = get_filtered_events(&cached_data);
    let new_events = get_new_events(&current, &cached);

    let mut matched_cache = load_matched_cache();
    let mut new_matches = Vec::new();

    if !new_events.is_empty() {
        info!("Found {} new event(s)", new_events.len());

        for mut event in new_events {
            debug!("Processing NOTAM id {}", event.notam_id);
            match fetch_detail_page(&client, &event.notam_id).await {
                Ok(html) => {
                    let parsed = parse_notam_html(&html);
                    event.parsed = Some(parsed.clone());

                    let searchable_text = format!(
                        "{} {} {} {} {} {} {} {} {}",
                        parsed.notam_id.clone().unwrap_or_default(),
                        parsed.location,
                        parsed.reason,
                        parsed.begin,
                        parsed.end,
                        parsed.restrictions,
                        parsed.other_info,
                        parsed.airspace.center,
                        parsed.airspace.altitude
                    )
                    .to_lowercase();

                    if keywords.is_empty() || keywords.iter().any(|kw| searchable_text.contains(kw))
                    {
                        info!("Event matches criteria: {}", event.notam_id);
                        new_matches.push(event.clone());
                        matched_cache.push(event.clone());
                    }
                }
                Err(e) => error!("Error processing event {}: {}", event.notam_id, e),
            }
        }
    }

    save_raw_cache(&current)?;
    save_matched_cache(&matched_cache)?;
    info!("Caches updated");

    if notify {
        if new_matches.is_empty() {
            info!("No matching events");
        } else if new_matches.len() == 1 {
            notify_event(&new_matches[0]);
        } else {
            notify_batch_event(&new_matches);
        }
    }

    Ok(new_matches)
}

pub async fn run_monitor(keywords: &[String]) -> Result<()> {
    let _ = process_feed(keywords, true).await?;
    Ok(())
}

pub async fn check_feed() -> Result<Vec<TFREvent>> {
    let keywords = load_keywords(None);
    process_feed(&keywords, false).await
}

/// Optional periodic runner (equivalent to main loop) that repeats monitoring every interval minutes.
pub async fn run_periodic(keywords: Vec<String>, interval_minutes: u64) -> Result<()> {
    loop {
        run_monitor(&keywords).await?;
        info!("Waiting {} minute(s)...", interval_minutes);
        tokio::time::sleep(Duration::from_secs(interval_minutes * 60)).await;
    }
}

fn extract_text(element: &scraper::ElementRef) -> String {
    element
        .text()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parses detailed NOTAM HTML into structured NotamDetail.
pub fn parse_notam_html(html_text: &str) -> TFREventDetail {
    let document = Html::parse_document(html_text);
    let table_selector = Selector::parse("table").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut detail = TFREventDetail::default();

    for table in document.select(&table_selector) {
        let table_text = extract_text(&table);

        // Metadata Table (Issue Date, NOTAM ID, etc.)
        if table_text.contains("Issue Date") {
            if table_text.contains("NOTAM Number") {
                if let Some(first_row) = table.select(&tr_selector).next() {
                    let tds: Vec<_> = first_row.select(&td_selector).collect();
                    if let Some(last) = tds.last() {
                        detail.notam_id = Some(extract_text(&last));
                    }
                }
            }

            for row in table.select(&tr_selector) {
                let row_text = extract_text(&row);
                let tds: Vec<_> = row.select(&td_selector).collect();
                if tds.is_empty() {
                    continue;
                }

                if row_text.contains("Issue Date") {
                    detail.issue_date = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Location") {
                    detail.location = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Beginning Date") {
                    detail.begin = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Ending Date") {
                    detail.end = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Reason") {
                    detail.reason = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Type") {
                    detail.r#type = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Replaced NOTAM") {
                    detail.replaced = extract_text(tds.last().unwrap());
                }
            }
        }

        // Airspace Definition Table
        if table_text.contains("Airspace Definition") {
            for row in table.select(&tr_selector) {
                let row_text = extract_text(&row);
                let tds: Vec<_> = row.select(&td_selector).collect();
                if tds.is_empty() {
                    continue;
                }

                if row_text.contains("Center:") {
                    detail.airspace.center = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Radius:") {
                    detail.airspace.radius = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Altitude:") {
                    detail.airspace.altitude = extract_text(tds.last().unwrap());
                }
                if row_text.contains("Effective Date") {
                    detail
                        .airspace
                        .effective
                        .push(extract_text(tds.last().unwrap()));
                }
            }
        }

        // Operating Restrictions and Requirements Table
        if table_text.contains("Operating Restrictions and Requirements") {
            detail.restrictions = table_text.clone();
        }

        // Other Information Table
        if table_text.contains("Other Information") {
            detail.other_info = table_text.clone();
        }
    }

    detail
}

/// Summarizes matched events for today
/// Returns (today_matches_count, unique_cities_count).
pub fn summarize_matched_events(events: &[TFREvent]) -> (usize, usize) {
    let today = chrono::Utc::now().date_naive();
    let mut cities = HashSet::new();
    let mut today_count = 0;

    for e in events {
        if let Some(parsed) = &e.parsed {
            if let Ok(dt) = chrono::NaiveDate::parse_from_str(&parsed.issue_date, "%m/%d/%Y") {
                if dt == today {
                    today_count += 1;
                    if !parsed.location.is_empty() {
                        cities.insert(parsed.location.clone());
                    }
                }
            }
        }
    }

    (today_count, cities.len())
}

use chrono::{DateTime, NaiveDateTime, Utc};

pub fn load_matched_cache_sorted() -> Vec<TFREvent> {
    let mut events = load_matched_cache();
    // Sort newest first by issue_date or fallback
    events.sort_by(|a, b| {
        let a_date = a.parsed.as_ref().and_then(|p| parse_date(&p.issue_date));
        let b_date = b.parsed.as_ref().and_then(|p| parse_date(&p.issue_date));
        b_date.cmp(&a_date)
    });
    events
}

fn parse_date(date_str: &str) -> Option<DateTime<Utc>> {
    // FAA format looks like "03/16/2025 13:00 UTC"
    if let Ok(ndt) = NaiveDateTime::parse_from_str(date_str, "%m/%d/%Y %H:%M %Z") {
        Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
    } else {
        None
    }
}
