use anyhow::{anyhow, Result};
use fantoccini::{Client, Locator};
use image::{GenericImageView, Rgba};
use serde_json::Value;
use std::time::Duration;
use tokio::try_join;

pub struct JobScraper {
    client: Client,
}

impl JobScraper {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn scrape(&self) -> Result<()> {
        for iteration in 1..=10 {
            println!("Starting iteration {} of 10", iteration);

            match self.scrape_single_iteration().await {
                Ok(_) => println!("Iteration {} completed successfully", iteration),
                Err(e) => {
                    println!(
                        "Error in iteration {}: {}. Attempting to recover...",
                        iteration, e
                    );
                    self.handle_unexpected_scenario().await?;
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        Ok(())
    }

    async fn scrape_single_iteration(&self) -> Result<()> {
        // self.login("anthonyc.animba@gmail.com", "Ar$enal.27")
        //     .await?;
        self.verify_login().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.click_discovery_card().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.click_first_qualifying_li().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.click_all_filters_button().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.select_advanced_filter().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.click_show_results_button().await?;
        self.find_clickable_apply_button().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.click_apply_button().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.click_modal_primary_button().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.click_next_button_in_modal().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.handle_review_and_submit().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let should_continue = self.handle_sponsorship_question().await?;
        if !should_continue {
            println!("Application process complete or sponsorship question not found.");
            self.take_screenshot("process_complete_screenshot.png")
                .await?;
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.take_screenshot("post_filters_click_screenshot.png")
            .await?;
        self.print_current_url().await?;
        self.print_page_title().await?;
        Ok(())
    }

    async fn handle_unexpected_scenario(&self) -> Result<()> {
        let dismiss_script = r#"
        function findAndClickDismissButton() {
            const selectors = [
                'button[aria-label="Dismiss"][data-test-modal-close-btn]',
                'button[aria-label="Dismiss"]',
                'button.artdeco-modal__dismiss',
                'button.artdeco-button--circle[aria-label="Dismiss"]',
                'button.artdeco-button--circle.artdeco-button--muted',
                'button[data-test-modal-close-btn]'
            ];
    
            for (const selector of selectors) {
                const button = document.querySelector(selector);
                if (button) {
                    button.click();
                    return `Dismiss button clicked using selector: ${selector}`;
                }
            }
    
            // If no button found, try to find by text content
            const buttons = document.querySelectorAll('button');
            for (const button of buttons) {
                if (button.textContent.trim().toLowerCase() === 'dismiss') {
                    button.click();
                    return 'Dismiss button clicked by text content';
                }
            }
    
            return "Dismiss button not found";
        }
    
        return findAndClickDismissButton();
        "#;

        let result: Value = self.client.execute(dismiss_script, vec![]).await?;
        println!("Dismiss button result: {:?}", result);

        tokio::time::sleep(Duration::from_secs(2)).await;

        let close_incomplete_script = r#"
            const closeButton = document.querySelector('button.artdeco-modal__confirm-dialog-btn');
            if (closeButton) {
                closeButton.click();
                return "Close button clicked";
            }
            return "Close button not found";
        "#;

        let result: Value = self.client.execute(close_incomplete_script, vec![]).await?;
        println!("Close button result: {:?}", result);

        Ok(())
    }

    async fn click_modal_primary_button(&self) -> Result<()> {
        let script = r#"
            function clickModalPrimaryButton() {
                const modal = document.querySelector('div.artdeco-modal');
                if (!modal) {
                    return "Modal not found";
                }
                
                const footer = modal.querySelector('footer');
                if (!footer) {
                    return "Modal footer not found";
                }

                const primaryButton = footer.querySelector('button.artdeco-button--primary');
                if (!primaryButton) {
                    return "Primary button not found in modal footer";
                }
                
                primaryButton.click();
                return "Successfully clicked the primary button in modal footer";
            }
            return clickModalPrimaryButton();
        "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        // async fn take_and_save_screenshot(client: &WebDriver, filename: &str) -> Result<()> {
        //     let screenshot = client.screenshot(ScreenshotOptions::default()).await?;
        //     std::fs::write(filename, &screenshot)?;
        //     println!("Screenshot saved as {}", filename);
        //     Ok(())
        // }

        match result.as_str() {
            Some("Successfully clicked the primary button in modal footer") => {
                println!("Successfully clicked the primary button in modal footer");
                // take_and_save_screenshot(&self.client, "success_screenshot.png").await?;
                Ok(())
            }
            Some(error_message) => {
                println!("Failed to click primary button: {}", error_message);
                // take_and_save_screenshot(&self.client, "error_screenshot.png").await?;
                Err(anyhow!("Failed to click primary button: {}", error_message))
            }
            None => {
                println!("Unexpected result from JavaScript execution");
                // take_and_save_screenshot(&self.client, "unexpected_screenshot.png").await?;
                Err(anyhow!("Unexpected result from JavaScript execution"))
            }
        }
    }

    async fn login(&self, username: &str, password: &str) -> Result<()> {
        self.client.goto("https://www.linkedin.com/jobs").await?;
        let username_field = self
            .client
            .find(Locator::Css("input[name='session_key']"))
            .await?;
        username_field.send_keys(username).await?;
        let password_field = self
            .client
            .find(Locator::Css("input[name='session_password']"))
            .await?;
        password_field.send_keys(password).await?;
        let submit_button = self
            .client
            .find(Locator::Css("button[type='submit']"))
            .await?;
        submit_button.click().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        Ok(())
    }

    async fn verify_login(&self) -> Result<bool> {
        let (profile, url, form, message) = try_join!(
            self.check_user_profile(),
            self.check_url(),
            self.check_login_form_absence(),
            self.check_welcome_message()
        )?;

        if profile || url || form || message {
            println!("Login successful!");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn check_user_profile(&self) -> Result<bool> {
        Ok(self
            .client
            .find(Locator::Css(".user-profile"))
            .await
            .is_ok())
    }

    async fn check_url(&self) -> Result<bool> {
        let current_url = self.client.current_url().await?;
        Ok(current_url
            .as_ref()
            .starts_with("https://www.linkedin.com/feed/"))
    }

    async fn check_login_form_absence(&self) -> Result<bool> {
        Ok(self.client.find(Locator::Css("form#login")).await.is_err())
    }

    async fn check_welcome_message(&self) -> Result<bool> {
        let body_text = self.client.find(Locator::Css("body")).await?.text().await?;
        Ok(body_text.contains("Welcome") || body_text.contains("Dashboard"))
    }

    async fn click_discovery_card(&self) -> Result<()> {
        self.click_element(".discovery-templates-jump-back-in-card")
            .await
    }

    async fn click_element(&self, fallback_selector: &str) -> Result<()> {
        let (x, y) = (814, 745);
        let js_code = format!(
            r#"
            function clickAt(x, y) {{
                const element = document.elementFromPoint(x, y);
                if (element) {{
                    const clickEvent = new MouseEvent('click', {{
                        view: window,
                        bubbles: true,
                        cancelable: true,
                        clientX: x,
                        clientY: y
                    }});
                    element.dispatchEvent(clickEvent);
                    return 'coordinate_click';
                }}
                const fallbackElement = document.querySelector('{}');
                if (fallbackElement) {{
                    fallbackElement.click();
                    return 'selector_click';
                }}
                return 'click_fail';
            }}
            return clickAt({}, {});
            "#,
            fallback_selector, x, y
        );

        let before_screenshot = self.client.screenshot().await?;
        std::fs::write("before_click_screenshot.png", &before_screenshot)?;
        println!("Before-click screenshot saved as before_click_screenshot.png");

        let result: Value = self.client.execute(&js_code, vec![]).await?;

        match result.as_str() {
            Some("coordinate_click") => println!("Clicked using coordinates ({}, {})", x, y),
            Some("selector_click") => {
                println!("Clicked using fallback selector: {}", fallback_selector)
            }
            _ => return Err(anyhow!("Failed to click element")),
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let after_screenshot = self.client.screenshot().await?;
        std::fs::write("after_click_screenshot.png", &after_screenshot)?;
        println!("After-click screenshot saved as after_click_screenshot.png");

        Ok(())
    }

    async fn click_first_qualifying_li(&self) -> Result<()> {
        let script = r#"
            const ul = document.querySelector("ul.artdeco-carousel__slider");
            if (!ul) return "UL not found";
            
            const lis = ul.querySelectorAll("li.artdeco-carousel__item");
            for (const li of lis) {
                const innerLi = li.querySelector("li.discovery-templates-jump-back-in-card");
                if (innerLi) {
                    const link = innerLi.querySelector("a.app-aware-link");
                    if (link) {
                        link.click();
                        return "Clicked qualifying LI";
                    }
                }
            }
            return "No qualifying LI found";
        "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Clicked qualifying LI") => {
                println!("Successfully clicked on the first qualifying li element");
                Ok(())
            }
            Some("No qualifying LI found") => Err(anyhow!("No qualifying li element found")),
            Some("UL not found") => Err(anyhow!(
                "UL with class 'artdeco-carousel__slider' not found"
            )),
            _ => Err(anyhow!("Unexpected result when clicking li element")),
        }
    }

    async fn click_all_filters_button(&self) -> Result<()> {
        let script = r#"
            const buttons = Array.from(document.querySelectorAll('button'));
            const allFiltersButton = buttons.find(button => button.textContent.trim() === 'All filters');
            if (allFiltersButton) {
                allFiltersButton.click();
                return true;
            }
            return false;
        "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_bool() {
            Some(true) => {
                println!("Successfully clicked 'All filters' button");
                Ok(())
            }
            _ => Err(anyhow!("Failed to find or click 'All filters' button")),
        }
    }

    async fn select_advanced_filter(&self) -> Result<()> {
        let script = r#"
            const container = document.querySelector('.search-reusables__secondary-filters-filter');
            if (!container) return "Container not found";
            
            const radioInput = container.querySelector('input[id="advanced-filter-sortBy-DD"]');
            if (!radioInput) return "Radio input not found";
            
            radioInput.click();
            return "Successfully clicked the advanced filter radio input";
        "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Successfully clicked the advanced filter radio input") => {
                println!("Successfully selected the advanced filter");
                Ok(())
            }
            Some(error_message) => Err(anyhow!(
                "Failed to select advanced filter: {}",
                error_message
            )),
            None => Err(anyhow!("Unexpected result from JavaScript execution")),
        }
    }

    async fn click_show_results_button(&self) -> Result<()> {
        let script = r#"
            const actionbar = document.querySelector('.artdeco-modal__actionbar');
            if (!actionbar) return "Actionbar not found";
            
            const showResultsButton = actionbar.querySelector('button.search-reusables__secondary-filters-show-results-button');
            if (!showResultsButton) return "Show results button not found";
            
            showResultsButton.click();
            return "Successfully clicked the show results button";
        "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Successfully clicked the show results button") => {
                println!("Successfully clicked the show results button");
                Ok(())
            }
            Some(error_message) => Err(anyhow!(
                "Failed to click show results button: {}",
                error_message
            )),
            None => Err(anyhow!("Unexpected result from JavaScript execution")),
        }
    }

    async fn find_clickable_apply_button(&self) -> Result<()> {
        let script = r#"
        async function findClickableApplyButton() {
            const isApplyButtonClickable = () => {
                const applyButton = document.querySelector('.job-details-jobs-unified-top-card__container--two-pane button.jobs-apply-button');
                return applyButton && !applyButton.disabled;
            };

            if (isApplyButtonClickable()) {
                return "Apply button is already clickable";
            }

            const jobList = document.querySelector('ul.scaffold-layout__list-container');
            if (!jobList) {
                return "Job list not found";
            }

            const jobItems = jobList.querySelectorAll('li');
            for (let item of jobItems) {
                item.click();
                await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for 1 second
                if (isApplyButtonClickable()) {
                    return "Found clickable apply button";
                }
            }

            return "No clickable apply button found";
        }
        return findClickableApplyButton();
        "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Apply button is already clickable") | Some("Found clickable apply button") => {
                println!("Apply button is clickable");
                Ok(())
            }
            Some("No clickable apply button found") => {
                println!("No job with a clickable apply button found. You may need to load more results or adjust your search.");
                Ok(())
            }
            Some("Job list not found") => Err(anyhow!("Could not find the job list container")),
            Some(error_message) => Err(anyhow!("Error: {}", error_message)),
            None => Err(anyhow!("Unexpected result from JavaScript execution")),
        }
    }

    async fn click_apply_button(&self) -> Result<()> {
        let script = r#"
            function clickApplyButton() {
                const container = document.querySelector('.job-details-jobs-unified-top-card__container--two-pane');
                if (!container) {
                    return "Job details container not found";
                }
                
                const applyButton = container.querySelector('button.jobs-apply-button');
                if (!applyButton) {
                    return "Apply button not found";
                }
                
                applyButton.click();
                return "Successfully clicked the apply button";
            }
            return clickApplyButton();
        "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Successfully clicked the apply button") => {
                println!("Successfully clicked the apply button");
                Ok(())
            }
            Some(error_message) => Err(anyhow!("Failed to click apply button: {}", error_message)),
            None => Err(anyhow!("Unexpected result from JavaScript execution")),
        }
    }

    async fn click_next_button_in_modal(&self) -> Result<()> {
        let script = r#"
                function clickNextButtonInModal() {
                    const modal = document.querySelector('div.artdeco-modal');
                    if (!modal) {
                        return "Modal not found";
                    }
                    
                    const footer = modal.querySelector('footer');
                    if (!footer) {
                        return "Modal footer not found";
                    }
    
                    const nextButton = Array.from(footer.querySelectorAll('button.artdeco-button--primary'))
                        .find(button => button.textContent.trim().toLowerCase() === 'next');
                    
                    if (!nextButton) {
                        return "Next button not found in modal footer";
                    }
                    
                    nextButton.click();
                    return "Successfully clicked the Next button in modal footer";
                }
                return clickNextButtonInModal();
            "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Successfully clicked the Next button in modal footer") => {
                println!("Successfully clicked the Next button in modal footer");
                Ok(())
            }
            Some("Next button not found in modal footer") => {
                println!("Next button not found. The application process might be complete.");
                Ok(())
            }
            Some(error_message) => Err(anyhow!("Error: {}", error_message)),
            None => Err(anyhow!("Unexpected result from JavaScript execution")),
        }
    }

    async fn handle_sponsorship_question(&self) -> Result<bool> {
        let script = r#"
                function handleSponsorshipQuestion() {
                    const questionLegend = Array.from(document.querySelectorAll('legend'))
                        .find(legend => legend.textContent.includes('Will you now or in the future require sponsorship for employment visa status?'));
                    
                    if (!questionLegend) {
                        return "Question not found";
                    }
                    
                    const formElement = questionLegend.closest('.radio-button-form-component-formElement-urn-li-jobs-applyformcommon-easyApplyFormElement-4048251551-5319829281-multipleChoice');
                    if (!formElement) {
                        return "Form element not found";
                    }
                    
                    const yesRadio = formElement.querySelector('input[value="Yes"]');
                    if (!yesRadio) {
                        return "Yes option not found";
                    }
                    
                    yesRadio.click();
                    return "Successfully selected 'Yes' for sponsorship question";
                }
                return handleSponsorshipQuestion();
            "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Successfully selected 'Yes' for sponsorship question") => {
                println!("Successfully handled sponsorship question");
                Ok(true)
            }
            Some("Question not found") => {
                println!("Sponsorship question not found. Ending process.");
                Ok(false)
            }
            Some(error_message) => Err(anyhow!("Error: {}", error_message)),
            None => Err(anyhow!("Unexpected result from JavaScript execution")),
        }
    }

    async fn handle_review_and_submit(&self) -> Result<()> {
        let script = r#"
            async function handleReviewAndSubmit() {
                function clickButtonIfExists(buttonText) {
                    const modal = document.querySelector('div.artdeco-modal');
                    if (!modal) return false;
                    
                    const footer = modal.querySelector('footer');
                    if (!footer) return false;
    
                    const button = Array.from(footer.querySelectorAll('button.artdeco-button--primary'))
                        .find(button => button.textContent.trim().toLowerCase() === buttonText.toLowerCase());
                    
                    if (button) {
                        button.click();
                        return true;
                    }
                    return false;
                }
    
                if (clickButtonIfExists('Review')) {
                    await new Promise(resolve => setTimeout(resolve, 2000)); // Wait for 2 seconds
                    if (clickButtonIfExists('Submit application')) {
                        return "Successfully reviewed and submitted application";
                    } else {
                        return "Reviewed but couldn't find submit button";
                    }
                }
                
                return "Review button not found";
            }
            return handleReviewAndSubmit();
            "#;

        let result: Value = self.client.execute(script, vec![]).await?;

        match result.as_str() {
            Some("Successfully reviewed and submitted application") => {
                println!("Application reviewed and submitted");
                Ok(())
            }
            Some("Reviewed but couldn't find submit button") => Err(anyhow!(
                "Application was reviewed but submit button was not found"
            )),
            Some("Review button not found") => {
                println!("Review button not found, continuing with the process");
                Ok(())
            }
            Some(error_message) => Err(anyhow!("Error: {}", error_message)),
            None => Err(anyhow!("Unexpected result from JavaScript execution")),
        }
    }

    async fn take_screenshot(&self, filename: &str) -> Result<()> {
        let screenshot = self.client.screenshot().await?;
        std::fs::write(filename, &screenshot)?;
        println!("Screenshot saved as {}", filename);
        Ok(())
    }

    async fn print_current_url(&self) -> Result<()> {
        let current_url = self.client.current_url().await?;
        println!("Current URL: {:?}", current_url);
        Ok(())
    }

    async fn print_page_title(&self) -> Result<()> {
        let title = self.client.title().await?;
        println!("Page title: {:?}", title);
        Ok(())
    }

    fn find_verify_button(screenshot_path: &str) -> Result<(u32, u32)> {
        let img = image::open(screenshot_path)?;
        let button_color = Rgba([0, 0, 0, 255]); // Black text color
        let (width, height) = img.dimensions();
        for y in (height / 2)..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                if pixel == button_color {
                    if Self::is_verify_text(&img, x, y) {
                        return Ok((x + 30, y + 15)); // Adjust these offsets as needed
                    }
                }
            }
        }
        Err(anyhow!("Could not find 'Verify' button in the screenshot"))
    }

    fn is_verify_text(img: &image::DynamicImage, x: u32, y: u32) -> bool {
        let expected_colors = [
            (0, 0),
            (10, 0),
            (20, 0),
            (30, 0),
            (40, 0),
            (50, 0), // Approximate positions of 'Verify' letters
        ];

        expected_colors.iter().all(|(dx, dy)| {
            let pixel = img.get_pixel(x + dx, y + dy);
            pixel == Rgba([0, 0, 0, 255]) // Black text color
        })
    }
}
