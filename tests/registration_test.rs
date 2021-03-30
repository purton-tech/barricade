mod common;

use dotenv::dotenv;
use thirtyfour::prelude::*;

// let's set up the sequence of steps we want the browser to take
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn registration() -> WebDriverResult<()> {
    dotenv().ok();

    let driver = common::Config::new().get_driver().await?;

    driver.get("http://localhost:9095").await?;

    // Find element.
    let email_input = driver.find_element(By::Id("email")).await?;

    // Click the search button.
    let elem_button = driver
        .find_element(By::Css("button[type='submit']"))
        .await?;
    elem_button.click().await?;

    assert!(
        driver
            .page_source()
            .await?
            .contains("Invalid email or password"),
        true
    );

    let email = common::random_email();
    email_input.send_keys(&email).await?;
    let password_input = driver.find_element(By::Id("password")).await?;
    password_input.send_keys(&email).await?;

    // We shouldn't find the user
    assert!(
        driver
            .page_source()
            .await?
            .contains("Invalid email or password"),
        true
    );

    // Let's go and register
    driver.find_element(By::LinkText("Sign Up")).await?.click().await?;
    driver.find_element(By::Id("email")).await?.send_keys(&email).await?;
    driver.find_element(By::Id("password")).await?.send_keys(&email).await?;
    driver.find_element(By::Id("confirm_password")).await?.send_keys(&email).await?;
    driver.find_element(By::Css("button[type='submit']")).await?.click().await?;

    Ok(())
}
