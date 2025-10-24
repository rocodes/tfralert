use core::FeedItem;
use reqwest::Client;
use std::env;

pub async fn post(item: &FeedItem) -> anyhow::Result<()> {
    let base_url = env::var("MASTODON_BASE_URL")?;
    let token = env::var("MASTODON_ACCESS_TOKEN")?;
    let client = Client::new();

    let status = format!("{}\n{}", item.title, item.url);
    client
        .post(format!("{base_url}/api/v1/statuses"))
        .bearer_auth(token)
        .form(&[("status", &status)])
        .send()
        .await?;

    Ok(())
}
