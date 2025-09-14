use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::config::get_config;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cfg = get_config().expect("failed to read config");
    let listener = TcpListener::bind(cfg.service.host).expect("failed to bind random port");
    let database = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database.conn_str())
        .await
        .expect("failed to connect to Postgres");

    run(listener, database)?.await
}
