//! Integration tests — hit the real API with the account from `.env`.
//! Run with: cargo test -p alpha-progression -- --ignored --nocapture

fn var(name: &str) -> String {
    dotenvy::dotenv().ok();
    std::env::var(name)
        .unwrap_or_else(|_| panic!("{name} is not set — see .env.example"))
}

async fn login() -> alpha_progression::Session {
    let client = alpha_progression::AlphaProgressionClient::new();
    client
        .login(&var("AP_TEST_EMAIL"), &var("AP_TEST_PASSWORD"))
        .await
        .expect("login failed")
}

#[tokio::test]
#[ignore]
async fn test_login_returns_known_uuid() {
    let session = login().await;
    assert_eq!(session.uuid, var("AP_TEST_UUID"));
    assert!(!session.token.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_check_subscription() {
    let client = alpha_progression::AlphaProgressionClient::new();
    let session = login().await;
    let subscribed = client.check_subscription(&session).await.expect("check failed");
    assert!(!subscribed, "test account should not be subscribed");
}

#[tokio::test]
#[ignore]
async fn test_db_info() {
    let client = alpha_progression::AlphaProgressionClient::new();
    let session = login().await;
    let info = client.db_info(&session).await.expect("db_info failed");
    assert_eq!(info.db_name, session.uuid);
}

#[tokio::test]
#[ignore]
async fn test_get_changes() {
    let client = alpha_progression::AlphaProgressionClient::new();
    let session = login().await;
    let changes = client
        .get_changes(&session, "0", 100)
        .await
        .expect("get_changes failed");
    println!("last_seq: {}", changes.last_seq);
    println!("doc count: {}", changes.results.len());
}
