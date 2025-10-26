// Copyright 2025. This file is part of TFRAlert.

// TFRAlert is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

// TFRAlert is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along with TFRAlert. If not, see <https://www.gnu.org/licenses/>.

use chrono::Utc;
use reqwest::blocking::Client;
use serde_json::json;
use std::env;

const BSKY_HANDLE: &str = "TODO";

// See https://docs.bsky.app/blog/create-post
// we just use the accessJwt, which is short-lived, but that's okay
// for a single post
pub async fn post(text: &String) -> anyhow::Result<()> {
    let app_password = env::var("BLUESKY_APP_PASSWORD").expect("Need Bluesky app password");

    let client = Client::new();

    let session_resp = client
        .post("https://bsky.social/xrpc/com.atproto.server.createSession")
        .json(&json!({
            "identifier": &BSKY_HANDLE,
            "password": app_password,
        }))
        .send()?
        .error_for_status()?;

    let session: serde_json::Value = session_resp.json()?;
    let access_jwt = session["accessJwt"].as_str().unwrap();
    let did = session["did"].as_str().unwrap();

    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let post = json!({
        "$type": "app.bsky.feed.post",
        "text": text,
        "createdAt": now,
    });

    let response = client
        .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
        .bearer_auth(access_jwt)
        .json(&json!({
            "repo": did,
            "collection": "app.bsky.feed.post",
            "record": post,
        }))
        .send()?
        .error_for_status()?;

    if response.status().is_success() {
        let body = response.json::<serde_json::Value>()?;
        println!("{}", serde_json::to_string_pretty(&body)?);
    } else {
        eprintln!("Error posting: {}", response.status());
    }

    Ok(())
}
