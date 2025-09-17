use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::config::get_config;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let cfg = get_config().expect("failed to read config");
    let listener = TcpListener::bind(cfg.service.host).expect("failed to bind random port");
    let database = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(cfg.database.with_db())
        .await
        .expect("failed to connect to Postgres");

    run(listener, database)?.await
}
