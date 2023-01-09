use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn page_creation_works() {
    let app = spawn_app().await;

    // Try creating page without login
    let response = app.create_page("Test Page").await;
    assert_is_redirect_to(&response, "/login");

    // Log in
    let response = app.test_user.login(&app).await;
    // TODO: this should at some point go back to the initial page
    assert_is_redirect_to(&response, "/admin");

    // Create page
    let response = app.create_page("Test Page").await;
    assert_is_redirect_to(&response, "/pages/Test Page");
    let page = sqlx::query!("SELECT title FROM page")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved page.");
    assert_eq!(page.title, "Test Page");
}

#[tokio::test]
async fn page_saving_works() {
    let app = spawn_app().await;

    // Log in
    let response = app.test_user.login(&app).await;
    assert_is_redirect_to(&response, "/admin");

    // Create page
    let create_response = app.create_page("Test Page").await;
    assert_is_redirect_to(&create_response, "/pages/Test Page");

    // Save edit
    let save_response = app.api_client
        .post(&format!("{}/pages/Test%20Page/edit", &app.address))
        .form(&[("markdown", "This is the test page summary.")])
        .send()
        .await
        .expect("Failed to execute page saving request.");

    // Assert
    assert_is_redirect_to(&save_response, "/pages/Test Page");
    let page = sqlx::query!("SELECT title, summary, content FROM page")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved page.");
    assert_eq!(page.title, "Test Page");
    assert_eq!(page.summary, Some("This is the test page summary.".to_string()));
    assert_eq!(page.content, "This is the test page summary.");
}