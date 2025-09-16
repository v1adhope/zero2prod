use once_cell::sync::Lazy;
use reqwest::StatusCode;
use secrecy::ExposeSecret;
use serde::Serialize;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    config::{Database, get_config},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub host: String,
    pub database: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut cfg = get_config().expect("failed to read config");
    cfg.database.database_name = Uuid::now_v7().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let host = format!("http://127.0.0.1:{}", port);

    let database = configure_database(&cfg.database).await;

    let srv = zero2prod::startup::run(listener, database.clone()).expect("failed to bind address");
    tokio::spawn(srv);

    TestApp {
        host: host,
        database: database,
    }
}

pub async fn configure_database(cfg: &Database) -> PgPool {
    let mut conn = PgConnection::connect(&cfg.conn_str_without_db().expose_secret())
        .await
        .expect("failed to connect to Postgres");
    conn.execute(format!(r#"create database "{}""#, cfg.database_name).as_str())
        .await
        .expect("failed to create database");

    let pool = PgPool::connect(&cfg.conn_str().expose_secret())
        .await
        .expect("failed to connect to Postgres");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to migrate the database");

    pool
}

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
        .fetch_one(&app.database)
        .await
        .expect("failed to fetch saved subscription");

    assert_eq!(saved.name, sub.name);
    assert_eq!(saved.email, sub.email);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
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
