pub mod common;

use thirtyfour::prelude::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore] // Doesn't work well in github actions
async fn account_enumeration_login() -> WebDriverResult<()> {
 
    let config = common::Config::new().await;

    let driver = config.get_driver().await?;
    let delay = std::time::Duration::new(11, 0);
    driver.set_implicit_wait_timeout(delay).await?;

    driver.get(&config.host).await?;

    // Register someone
    let email = common::register_random_user(&driver).await?;

    // Sign out then go to sign in page
    driver
        .get(format!("{}/auth/sign_out", &config.host))
        .await?;

    // Try and log in as a user that doens't exist
    driver
        .find_element(By::Id("email"))
        .await?
        .send_keys(&email)
        .await?;
    driver
        .find_element(By::Id("password"))
        .await?
        .send_keys("the-wrong-password")
        .await?;
    driver
        .find_element(By::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Look for the class to implicitly wait for the page to load.
    driver.find_element(By::ClassName("error")).await?;
    let no_user_page_source = driver.page_source().await?;
    assert!(no_user_page_source.contains("Invalid email or password"));

    // Try and log in as existing user with wrong password
    driver
        .find_element(By::Id("password"))
        .await?
        .send_keys("the-wrong-password")
        .await?;
    driver
        .find_element(By::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Look for the class to implicitly wait for the page to load.
    driver.find_element(By::ClassName("error")).await?;
    let existing_user_page_source = driver.page_source().await?;

    assert!(existing_user_page_source.contains("Invalid email or password"));

    // Make sure the returned content is exactly the same.
    assert_eq!(no_user_page_source, existing_user_page_source);

    // Always explicitly close the browser. There are no async destructors.
    driver.quit().await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore] // Doesn't work well in github actions
async fn account_enumeration_registration() -> WebDriverResult<()> {
    let config = common::Config::new().await;

    let driver = config.get_driver().await?;
    let delay = std::time::Duration::new(11, 0);
    driver.set_implicit_wait_timeout(delay).await?;

    driver.get(&config.host).await?;

    // Try and register as a user that doesn't exist
    let email = common::register_random_user(&driver).await?;

    // Wait for page to load as code might not be in database yet.
    driver.find_element(By::Id("code")).await?;

    let no_user_page_source = driver.page_source().await?;
    assert!(no_user_page_source.contains("Enter your confirmation code"));

    // Sign out then go to sign up page
    driver
        .get(format!("{}/auth/sign_out", &config.host))
        .await?;
    driver
        .find_element(By::LinkText("SIGN UP"))
        .await?
        .click()
        .await?;

    // Try and register as a user that does exist
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

    // Wait for page to load as code might not be in database yet.
    driver.find_element(By::Id("code")).await?;

    let existing_user_page_source = driver.page_source().await?;

    assert!(existing_user_page_source.contains("Enter your confirmation code"));

    // Make sure the returned content is exactly the same.
    assert_eq!(no_user_page_source, existing_user_page_source);

    // Always explicitly close the browser. There are no async destructors.
    driver.quit().await?;

    Ok(())
}
