// Copyright 2025. This file is part of TFRAlert.

// TFRAlert is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

// TFRAlert is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along with TFRAlert. If not, see <https://www.gnu.org/licenses/>.

use reqwest::Client;
use std::env;

pub async fn post(text: &String) -> anyhow::Result<()> {
    let base_url = env::var("MASTODON_BASE_URL").expect("Need mastodon base url");
    let token = env::var("MASTODON_ACCESS_TOKEN").expect("Need mastodon access token");
    let client = Client::new();

    client
        .post(format!("{base_url}/api/v1/statuses"))
        .bearer_auth(token)
        .form(&[("status", text)])
        .send()
        .await?;

    Ok(())
}
