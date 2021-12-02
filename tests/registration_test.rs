pub mod common;

use dotenv::dotenv;
use std::path::Path;
use thirtyfour::prelude::*;

// let's set up the sequence of steps we want the browser to take
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn registration() -> WebDriverResult<()> {
    dotenv().ok();

    let config = common::Config::new().await;

    let driver = config.get_driver().await?;
    let delay = std::time::Duration::new(11, 0);
    driver.set_implicit_wait_timeout(delay).await?;

    driver.get(&config.host).await?;
    //driver.get("http://localhost:9095").await?;

    // Click the logoin button with nothing in.
    let elem_button = driver
        .find_element(By::Css("button[type='submit']"))
        .await?;
    elem_button.click().await?;

    // Look for the class to implicitly wait for the page to load.
    driver.find_element(By::ClassName("error")).await?;

    assert!(driver
        .page_source()
        .await?
        .contains("Invalid email or password"));

    // Register someone
    let email = common::register_random_user(&driver).await?;

    // OTP Code
    // Wait for page to load as code might not be in database yet.
    driver.find_element(By::Id("code")).await?;

    let code = common::get_otp_code_from_database(&config)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    driver
        .find_element(By::Id("code"))
        .await?
        .send_keys(code.to_string())
        .await?;
    driver
        .find_element(By::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for page load
    driver
        .find_element(By::Css(
            "pre[style='word-wrap: break-word; white-space: pre-wrap;']",
        ))
        .await?;

    // Doesn't work in CI CD
    assert!(driver.page_source().await?.contains("User-Agent"));

    let cookie = driver.get_cookie("session").await;

    assert!(cookie.is_ok());

    driver
        .screenshot(Path::new("./target/registered.png"))
        .await?;

    driver
        .get(format!("{}/auth/sign_out", &config.host))
        .await?;

    let cookie = driver.get_cookie("session").await;

    assert!(cookie.is_err());

    // Lets log back in again.

    driver
        .find_element(By::Id("email"))
        .await?
        .send_keys(&email)
        .await?;
    driver
        .find_element(By::Id("password"))
        .await?
        .send_keys(&email)
        .await?;
    driver
        .find_element(By::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // OTP Code
    // Wait for page to load as code might not be in database yet.
    driver.find_element(By::Id("code")).await?;

    let code = common::get_otp_code_from_database(&config)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    driver
        .find_element(By::Id("code"))
        .await?
        .send_keys(code.to_string())
        .await?;
    driver
        .find_element(By::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for page load
    driver
        .find_element(By::Css(
            "pre[style='word-wrap: break-word; white-space: pre-wrap;']",
        ))
        .await?;

    // Always explicitly close the browser. There are no async destructors.
    driver.quit().await?;

    Ok(())
}
