// Copyright 2025. This file is part of TFRAlert.

// TFRAlert is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

// TFRAlert is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along with TFRAlert. If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
use log::{debug, error, info};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};

const RAW_EVENT_CACHE: &str = "tfr_cache.json";
const MATCHED_EVENT_CACHE: &str = "tfr_matches.json";

const JSON_FEED_TFR_URL: &str = "https://tfr.faa.gov/tfrapi/exportTfrList";
pub const NOTAM_DETAIL_URL: &str = "https://tfr.faa.gov/tfrapi/getWebText?notamId=";
// todo customization
const ALTITUDE_PARAMS: &str = "up to and including 400 feet AGL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTFREvent {
    pub notam_id: String,
    pub description: String,
    pub location: Option<String>,
    pub r#type: Option<String>,
    pub parsed: Option<ParsedTFREvent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedTFREvent {
    pub notam_id: String,
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
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Airspace {
    pub center: String,
    pub radius: String,
    pub altitude: String,
    pub effective: Vec<String>,
}

// Trait - they can both be represented as json
pub trait TFREvent: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug {}
impl TFREvent for ParsedTFREvent {}
impl TFREvent for RawTFREvent {}

pub async fn download_json_feed(client: &Client) -> Result<Vec<RawTFREvent>> {
    let resp = client.get(JSON_FEED_TFR_URL).send().await?;
    let json = resp.json::<Vec<RawTFREvent>>().await?;
    Ok(json)
}

pub fn load_raw_cache() -> Vec<RawTFREvent> {
    read_cache_file(RAW_EVENT_CACHE)
}

pub fn save_raw_cache(data: &[RawTFREvent]) -> Result<()> {
    write_cache_file(RAW_EVENT_CACHE, data)
}

pub fn load_matched_cache() -> Vec<ParsedTFREvent> {
    read_cache_file(MATCHED_EVENT_CACHE)
}

pub fn save_matched_cache(data: &[ParsedTFREvent]) -> Result<()> {
    write_cache_file(MATCHED_EVENT_CACHE, data)
}

fn read_cache_file<T: TFREvent>(path: &str) -> Vec<T> {
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

fn write_cache_file<T: TFREvent>(path: &str, data: &[T]) -> Result<()> {
    let serialized = serde_json::to_string_pretty(data)?;
    fs::write(path, serialized)?;
    Ok(())
}

/// event type "SECURITY"
pub fn get_filtered_events(data: &[RawTFREvent]) -> Vec<RawTFREvent> {
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
pub fn get_new_events(current: &[RawTFREvent], cached: &[RawTFREvent]) -> Vec<RawTFREvent> {
    let cached_ids: HashSet<_> = cached.iter().map(|e| &e.notam_id).collect();
    current
        .iter()
        .filter(|e| !cached_ids.contains(&e.notam_id))
        .cloned()
        .collect()
}

/// NOTAM detail page
pub async fn fetch_detail_page(client: &Client, notam_id: &str) -> Result<String> {
    let url = format!("{}{}", NOTAM_DETAIL_URL, notam_id);
    let resp = client.get(&url).send().await?.text().await?;
    Ok(resp)
}

/// keyword search from text file (TODO)
pub fn load_keywords(path: Option<&str>) -> Vec<String> {
    if let Some(path) = path {
        debug!("Load keywords...");
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

async fn process_feed(keywords: &[String]) -> Result<Vec<ParsedTFREvent>> {
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
                        "{} {} {} {} {} {} {} {} {} {}",
                        parsed.notam_id.clone(),
                        parsed.location,
                        parsed.reason,
                        parsed.begin,
                        parsed.end,
                        parsed.restrictions,
                        parsed.other_info,
                        parsed.airspace.center,
                        parsed.airspace.altitude,
                        parsed.description
                    )
                    .to_lowercase();

                    if keywords.is_empty()
                        || keywords.iter().any(|kw| {
                            searchable_text.contains(kw)
                                || searchable_text.contains(ALTITUDE_PARAMS)
                        })
                    {
                        info!("Event matches criteria: {}", event.notam_id);
                        new_matches
                            .push(event.parsed.as_ref().expect("Need parsed TFR data").clone());
                        matched_cache.push(event.parsed.expect("Need parsed TFR data").clone());
                    }
                }
                Err(e) => error!("Error processing event {}: {}", event.notam_id, e),
            }
        }
    }

    save_raw_cache(&current)?;
    save_matched_cache(&matched_cache)?;
    info!("Cache updated");
    Ok(new_matches)
}

pub async fn check_feed() -> Result<Vec<ParsedTFREvent>> {
    debug!("Check feed...");
    // TODO
    let keywords = load_keywords(None);
    process_feed(&keywords).await
}

fn extract_text(element: &scraper::ElementRef) -> String {
    element
        .text()
        .map(|t| t.trim().replace("\\r", "").replace("\\n", ""))
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parses detailed NOTAM HTML into structured NotamDetail.
pub fn parse_notam_html(html_text: &str) -> ParsedTFREvent {
    let document = Html::parse_document(html_text);
    let table_selector = Selector::parse("table").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut detail = ParsedTFREvent::default();

    for table in document.select(&table_selector) {
        let table_text = extract_text(&table);

        // Metadata table (date, notam id)
        if table_text.contains("Issue Date") {
            if table_text.contains("NOTAM Number") {
                if let Some(first_row) = table.select(&tr_selector).next() {
                    let tds: Vec<_> = first_row.select(&td_selector).collect();
                    if let Some(last) = tds.last() {
                        let font_selector = Selector::parse("font").unwrap();
                        if let Some(font) = last.select(&font_selector).next() {
                            detail.notam_id = extract_text(&font)
                                .replace("FDC", "")
                                .trim_ascii()
                                .to_string();
                        }
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

        // parse "airspace" table for altitude restrictions
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

        if table_text.contains("Operating Restrictions and Requirements") {
            detail.restrictions = table_text.clone();
        }

        if table_text.contains("Other Information") {
            detail.other_info = table_text.clone();
        }
    }

    detail
}

/// Summarize matched events (today_matches_count, unique_cities_count).
/// TODO (formating/parsing)
pub fn summarize_matched_events(events: &[ParsedTFREvent]) -> (usize, usize) {
    let today = chrono::Utc::now().date_naive();
    let mut cities = HashSet::new();
    let mut today_count = 0;

    for e in events {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(&e.issue_date, "%m/%d/%Y") {
            if dt == today {
                today_count += 1;
                if !e.location.is_empty() {
                    cities.insert(e.location.clone());
                }
            }
        }
    }
    (today_count, cities.len())
}

use chrono::{DateTime, NaiveDateTime, Utc};

pub fn load_matched_cache_sorted() -> Vec<ParsedTFREvent> {
    let mut events = load_matched_cache();
    /// TODO
    // events.sort_by(|a, b| {
    //     let a_date = a.as_ref().and_then(|p| parse_date(&p.issue_date));
    //     let b_date = b.as_ref().and_then(|p| parse_date(&p.issue_date));
    //     b_date.cmp(&a_date)
    // });
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

pub async fn refresh_tfr_results() -> Result<crate::FeedResult> {
    // TODO sorting by date or city/location
    let mut tfr_matches = load_matched_cache();
    let cached_match_count = tfr_matches.len();
    info!("{cached_match_count} cached");

    if let Ok(new_matches) = check_feed().await {
        // reverse to avoid oldest first
        for e in new_matches.iter().rev() {
            if !tfr_matches.iter().any(|m| m.notam_id == e.notam_id) {
                tfr_matches.insert(0, e.clone());
            }
        }
        save_matched_cache(&tfr_matches)?;
    }

    // FIXME (totals/ summaries)
    let (today_total, city_count) = summarize_matched_events(&tfr_matches);
    // let new_since_last = today_total - cached_match_count;

    Ok(crate::FeedResult {
        events: tfr_matches,
        unseen_count: 0, // todo
        today_count: today_total,
        city_today_count: city_count,
    })
}
