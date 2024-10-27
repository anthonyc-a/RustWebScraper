use anyhow::{anyhow, Result};
use fantoccini::{Client, ClientBuilder};
use serde_json::Value;
use std::time::Duration;

pub async fn connect_with_retry(url: &str, retries: u32, delay: Duration) -> Result<Client> {
    let mut caps = serde_json::map::Map::new();
    let chrome_opts = serde_json::json!({
        "args": ["--headless", "--disable-gpu"],
        "binary": "/Applications/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",  // Adjust this path
    });
    caps.insert("goog:chromeOptions".to_string(), chrome_opts);

    let mut attempt = 0;
    loop {
        match ClientBuilder::native()
            .capabilities(caps.clone())
            .connect(url)
            .await
        {
            Ok(client) => return Ok(client),
            Err(e) => {
                attempt += 1;
                if attempt >= retries {
                    return Err(e.into());
                }
                println!(
                    "Connection attempt {} failed, retrying in {:?}...",
                    attempt, delay
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

pub async fn execute_script(client: &Client, script: &str) -> Result<Value> {
    client.execute(script, vec![]).await.map_err(|e| anyhow!("Failed to execute script: {:?}", e))
}