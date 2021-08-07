mod common;

use dotenv::dotenv;
use std::path::Path;
use thirtyfour::prelude::*;

// let's set up the sequence of steps we want the browser to take
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn registration() -> WebDriverResult<()> {
    dotenv().ok();

    let config = common::Config::new();

    let driver = config.get_driver().await?;
    let delay = std::time::Duration::new(11, 0);
    driver.set_implicit_wait_timeout(delay).await?;

    driver.get(&config.host).await?;

    // Click the search button.
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

    let email = common::random_email();

    // Let's go and register
    driver
        .find_element(By::LinkText("SIGN UP"))
        .await?
        .click()
        .await?;
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
        .find_element(By::Id("confirm_password"))
        .await?
        .send_keys(&email)
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

    driver
        .find_element(By::Css("button[type='submit']"))
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

    // Wait for page load
    driver
        .find_element(By::Css(
            "pre[style='word-wrap: break-word; white-space: pre-wrap;']",
        ))
        .await?;

    Ok(())
}
