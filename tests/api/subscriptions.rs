use reqwest::StatusCode;
use serde::Serialize;

use crate::helpers::spawn_app;

#[derive(Serialize)]
struct Subscription {
    name: Option<String>,
    email: Option<String>,
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let sub = Subscription {
        name: Some("vla".to_string()),
        email: Some("vla@example.com".to_string()),
    };

    let resp = client
        .post(&format!("{}/subscriptions", &app.host))
        .form(&sub)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::OK, resp.status());

    let saved = sqlx::query!("select email, name from subscriptions")
        .fetch_one(&app.pg_pool)
        .await
        .expect("failed to fetch saved subscription");

    assert_eq!(saved.name, sub.name);
    assert_eq!(saved.email, sub.email);
}

#[tokio::test]
async fn subscribe_returns_a_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (
            "Missing email",
            Subscription {
                name: Some("vla".to_string()),
                email: None,
            },
        ),
        (
            "Missing name",
            Subscription {
                name: None,
                email: Some("vla@example.com".to_string()),
            },
        ),
        (
            "Missing whole data",
            Subscription {
                name: None,
                email: None,
            },
        ),
        (
            "Empty email",
            Subscription {
                name: Some("vla".to_string()),
                email: Some("".to_string()),
            },
        ),
        (
            "Empty name",
            Subscription {
                name: Some("".to_string()),
                email: Some("vla@example.com".to_string()),
            },
        ),
        (
            "Invalid email",
            Subscription {
                name: Some("vla".to_string()),
                email: Some("vla-example.com".to_string()),
            },
        ),
    ];

    for (name, input) in test_cases {
        let resp = client
            .post(&format!("{}/subscriptions", &app.host))
            .form(&input)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(StatusCode::BAD_REQUEST, resp.status(), "case name {}", name);
    }
}
