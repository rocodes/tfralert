// Copyright 2025. This file is part of TFRAlert.

// TFRAlert is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

// TFRAlert is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License along with TFRAlert. If not, see <https://www.gnu.org/licenses/>.

use anyhow;
use std::env;
use tfr_core::check_feed;
mod bsky;
mod mastodon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let items = check_feed().await?;

    // todo, stuff here like load from cache

    for item in items.iter().take(5) {
        mastodon::post(&item).await?;
        bksy::post(&item).await?;
    }

    Ok(())
}
