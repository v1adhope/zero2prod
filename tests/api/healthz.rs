use crate::helpers::spawn_app;

#[tokio::test]
async fn healthz_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/healthz", &app.host))
        .send()
        .await
        .expect("failed to execute request");

    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}
