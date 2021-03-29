use thirtyfour::prelude::*;

#[actix_rt::test]
async fn register() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--no-sandbox")?;
    caps.add_chrome_arg("--disable-gpu")?;
    caps.set_headless()?;
    let driver = WebDriver::new("http://localhost:4444", &caps).await?;

    // Navigate to https://wikipedia.org.
    driver.get("https://wikipedia.org").await?;

    Ok(())
}
