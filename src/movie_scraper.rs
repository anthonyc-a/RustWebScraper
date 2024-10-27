use crate::common::execute_script;
use anyhow::{anyhow, Result};
use fantoccini::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieInfo {
    title: String,
    year: Option<String>,
    quality: Option<String>,
    duration: Option<String>,
    poster_url: Option<String>,
}

pub struct MovieScraper {
    client: Client,
}

impl MovieScraper {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn scrape(&self) -> Result<()> {
        self.navigate_to_dopebox().await?;
        let movies = self.scrape_all_movies().await?;
        println!("Scraped {} movies", movies.len());
        // Here you can decide what to do with the scraped movies
        // For example, you could save them to a file or database
        self.save_movies_to_file(&movies, "scraped_movies.json")
            .await?;
        self.click_first_movie().await?;
        self.click_play_button().await?;
        self.take_screenshot("final_play_page_screenshot.png")
            .await?;
        Ok(())
    }

    async fn navigate_to_dopebox(&self) -> Result<()> {
        self.client.goto("https://dopebox.to/home").await?;
        println!("Navigated to Dopebox");
        self.take_screenshot("dopebox_home.png").await?;
        Ok(())
    }

    async fn scrape_all_movies(&self) -> Result<Vec<MovieInfo>> {
        let script = r#"
        function scrapeMovies() {
            const movieElements = document.querySelectorAll('.film_list-wrap .flw-item');
            return Array.from(movieElements).map(movie => {
                const titleElement = movie.querySelector('.film-poster-ahref');
                const yearElement = movie.querySelector('.fdi-item');
                const qualityElement = movie.querySelector('.pick.film-poster-quality');
                const durationElement = movie.querySelector('.fdi-duration');
                const posterElement = movie.querySelector('.film-poster-img');
                
                return {
                    title: titleElement ? titleElement.getAttribute('title') : 'Unknown',
                    year: yearElement ? yearElement.textContent.trim() : null,
                    quality: qualityElement ? qualityElement.textContent.trim() : null,
                    duration: durationElement ? durationElement.textContent.trim() : null,
                    poster_url: posterElement ? posterElement.getAttribute('data-src') : null
                };
            });
        }
        return JSON.stringify(scrapeMovies());
        "#;

        let result = execute_script(&self.client, script).await?;
        let movies: Vec<MovieInfo> = serde_json::from_str(&result.as_str().unwrap_or("[]"))?;

        self.take_screenshot("after_scraping_all_movies.png")
            .await?;

        Ok(movies)
    }

    async fn save_movies_to_file(&self, movies: &[MovieInfo], filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(movies)?;
        std::fs::write(filename, json)?;
        println!("Saved {} movies to {}", movies.len(), filename);
        Ok(())
    }

    async fn click_first_movie(&self) -> Result<()> {
        let script = r#"
        function clickFirstMovie() {
            const firstMovie = document.querySelector('.film_list-wrap .flw-item');
            if (!firstMovie) {
                return "No movie found";
            }
            
            const link = firstMovie.querySelector('a');
            if (!link) {
                return "No link found in the first movie item";
            }
            
            const href = link.getAttribute('href');
            if (!href) {
                return "No href attribute found in the link";
            }
            
            window.movieHref = href;
            link.click();
            
            return "Clicked on the first movie";
        }
        return clickFirstMovie();
        "#;

        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 5;

        while attempts < MAX_ATTEMPTS {
            attempts += 1;
            println!("Attempt {} to click on the first movie", attempts);

            let result = execute_script(&self.client, script).await?;
            self.take_screenshot(&format!("attempt_{}_before_click.png", attempts))
                .await?;

            match result.as_str() {
                Some("Clicked on the first movie") => {
                    println!("Successfully clicked on the first movie");

                    tokio::time::sleep(Duration::from_secs(2)).await;
                    self.take_screenshot(&format!("attempt_{}_after_click.png", attempts))
                        .await?;

                    self.close_other_tabs().await?;

                    if self.is_on_movie_page().await? {
                        self.take_screenshot(&format!("attempt_{}_success.png", attempts))
                            .await?;
                        return Ok(());
                    }
                }
                Some(error_msg) => {
                    println!("Error: {}", error_msg);
                    self.take_screenshot(&format!("attempt_{}_error.png", attempts))
                        .await?;
                }
                _ => {
                    println!("Unexpected result when clicking on the movie");
                    self.take_screenshot(&format!("attempt_{}_unexpected.png", attempts))
                        .await?;
                }
            }

            println!("Attempt {} failed, retrying...", attempts);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Err(anyhow!(
            "Failed to navigate to the movie page after {} attempts",
            MAX_ATTEMPTS
        ))
    }
    async fn click_play_button(&self) -> Result<()> {
        let script = r#"
        function clickPlayButton() {
            const playButton = document.querySelector('.btn-play');
            if (!playButton) {
                return "No play button found";
            }
            
            playButton.click();
            return "Clicked on the play button";
        }
        return clickPlayButton();
        "#;

        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 5;

        while attempts < MAX_ATTEMPTS {
            attempts += 1;
            println!("Attempt {} to click the play button", attempts);

            self.take_screenshot(&format!(
                "play_button_attempt_{}_before_click.png",
                attempts
            ))
            .await?;

            let result = execute_script(&self.client, script).await?;

            match result.as_str() {
                Some("Clicked on the play button") => {
                    println!("Successfully clicked the play button");

                    // Wait for potential ad popups
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    self.take_screenshot(&format!(
                        "play_button_attempt_{}_after_click.png",
                        attempts
                    ))
                    .await?;

                    // Close any newly opened tabs except the main one
                    self.close_other_tabs().await?;

                    // Check if we're on the video player page
                    if self.is_on_video_player_page().await? {
                        self.take_screenshot(&format!(
                            "play_button_attempt_{}_success.png",
                            attempts
                        ))
                        .await?;
                        return Ok(());
                    }
                }
                Some("No play button found") => {
                    println!("Play button not found, retrying...");
                    self.take_screenshot(&format!(
                        "play_button_attempt_{}_not_found.png",
                        attempts
                    ))
                    .await?;
                }
                _ => {
                    println!("Unexpected result when clicking the play button");
                    self.take_screenshot(&format!(
                        "play_button_attempt_{}_unexpected.png",
                        attempts
                    ))
                    .await?;
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        Err(anyhow!(
            "Failed to click the play button after {} attempts",
            MAX_ATTEMPTS
        ))
    }

    async fn close_other_tabs(&self) -> Result<()> {
        let handles = self.client.windows().await?;
        let current_handle = self.client.window().await?;

        for handle in handles {
            if handle != current_handle {
                self.client.switch_to_window(handle).await?;
                self.client.close_window().await?;
            }
        }

        self.client.switch_to_window(current_handle).await?;
        Ok(())
    }

    async fn is_on_movie_page(&self) -> Result<bool> {
        let script = r#"
    function checkMoviePage() {
        // Check if we're on the stored href
        if (window.movieHref && window.location.href.includes(window.movieHref)) {
            return true;
        }
        
        // Additional checks can be added here if needed
        // For example, checking for specific elements on the movie page
        
        return false;
    }
    return checkMoviePage();
    "#;

        let result = execute_script(&self.client, script).await?;
        Ok(result.as_bool().unwrap_or(false))
    }
    async fn is_on_video_player_page(&self) -> Result<bool> {
        let script = r#"
        function checkVideoPlayerPage() {
            // Check for elements typically found on a video player page
            const videoPlayer = document.querySelector('video');
            const playerControls = document.querySelector('.jw-controls');
            
            return !!(videoPlayer || playerControls);
        }
        return checkVideoPlayerPage();
        "#;

        let result = execute_script(&self.client, script).await?;
        Ok(result.as_bool().unwrap_or(false))
    }

    async fn take_screenshot(&self, filename: &str) -> Result<()> {
        let screenshot = self.client.screenshot().await?;
        std::fs::write(filename, &screenshot)?;
        println!("Screenshot saved as {}", filename);
        Ok(())
    }
}
