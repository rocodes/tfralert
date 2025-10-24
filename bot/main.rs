use std::env;
use core::fetch_feed;
mod mastodon;
mod bluesky;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let feed_url = env::var("FEED_URL")?;
    let items = fetch_feed(&feed_url).await?;

    for item in items.iter().take(5) {
        mastodon::post(&item).await?;
        bluesky::post(&item).await?;
    }

    Ok(())
}
