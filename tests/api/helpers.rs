use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    config::{Database, get_config},
    startup::App,
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
    pub pg_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut cfg = get_config().expect("failed to read config");

    cfg.database.database_name = Uuid::now_v7().to_string();
    let pg_pool = configure_database(&cfg.database).await;

    cfg.service.host = "127.0.0.1:0".to_string();
    let app = App::build(cfg).await.expect("failed to build app");
    let host = format!("http://127.0.0.1:{}", app.port());
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp { host, pg_pool }
}

async fn configure_database(cfg: &Database) -> PgPool {
    let mut conn = PgConnection::connect_with(&cfg.without_db())
        .await
        .expect("failed to connect to Postgres");
    conn.execute(format!(r#"create database "{}""#, cfg.database_name).as_str())
        .await
        .expect("failed to create database");

    let pool = PgPool::connect_with(cfg.with_db())
        .await
        .expect("failed to connect to Postgres");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to migrate the database");

    pool
}
