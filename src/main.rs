use anyhow::{anyhow, Result};
use fantoccini::Client;
use std::io::{self, Write};
use std::time::Duration;

mod common;
mod job_scraper;
mod movie_scraper;

use common::connect_with_retry;
use job_scraper::JobScraper;
use movie_scraper::MovieScraper;

fn prompt_user() -> Result<String> {
    println!("Which service would you like to use?");
    println!("1. Job Scraper");
    println!("2. Movie Scraper");
    print!("Enter your choice (1 or 2): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    Ok(choice.trim().to_string())
}

fn main() -> Result<()> {
    // Create a new runtime
    let runtime = tokio::runtime::Runtime::new()?;

    // Use the runtime to run our async main
    runtime.block_on(async {
        let client = connect_with_retry("http://localhost:9515", 5, Duration::from_secs(2)).await?;

        let choice = prompt_user()?;

        match choice.as_str() {
            "1" => {
                println!("Running Job Scraper...");
                let job_scraper = JobScraper::new(client.clone());
                job_scraper.scrape().await?;
            }
            "2" => {
                println!("Running Movie Scraper...");
                let movie_scraper = MovieScraper::new(client.clone());
                movie_scraper.scrape().await?;
            }
            _ => {
                return Err(anyhow!("Invalid choice. Please enter 1 or 2."));
            }
        }

        client.close().await?;

        Ok(())
    })
}
