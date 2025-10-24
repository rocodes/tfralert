use core::FeedItem;
use atproto::Client;
use std::env;

pub async fn post(item: &FeedItem) -> anyhow::Result<()> {
    let handle = env::var("BLUESKY_HANDLE")?;
    let password = env::var("BLUESKY_PASSWORD")?;
    let text = format!("{}\n{}", item.title, item.url);

    let mut client = Client::new(&handle, &password).await?;
    client.create_post(text).await?;
    Ok(())
}
