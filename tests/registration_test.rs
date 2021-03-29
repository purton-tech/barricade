mod common;

use dotenv::dotenv;
use thirtyfour::prelude::*;

// let's set up the sequence of steps we want the browser to take
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn registration()  -> WebDriverResult<()> {
    dotenv().ok();

    let config = common::Config::new();

    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--no-sandbox")?;
    caps.add_chrome_arg("--disable-gpu")?;
    caps.set_headless()?;
    let driver = WebDriver::new(&config.webdriver_url, &caps).await?;

    // Navigate to https://wikipedia.org.
    driver.get("https://wikipedia.org").await?;

    // Find element.
    let elem_form = driver.find_element(By::Id("search-form")).await?;

    // Find element from element.
    let elem_text = elem_form.find_element(By::Id("searchInput")).await?;

    // Type in the search terms.
    elem_text.send_keys("selenium").await?;

    // Click the search button.
    let elem_button = elem_form.find_element(By::Css("button[type='submit']")).await?;
    elem_button.click().await?;

    Ok(())
}

// chromedriver --whitelisted-ips=""