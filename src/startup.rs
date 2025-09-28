use crate::{
    config::Settings,
    email_client::EmailClient,
    routes::{healthz, subscribe},
};
use actix_web::{HttpServer, dev::Server, web};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{net::TcpListener, time::Duration};
use tracing_actix_web::TracingLogger;

pub struct App {
    port: u16,
    server: Server,
}

impl App {
    pub async fn build(cfg: Settings) -> Result<Self, std::io::Error> {
        let pg_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(cfg.database.with_db())
            .await
            .expect("failed to connect to Postgres");

        let sender = cfg
            .email_client
            .sender()
            .expect("invalid sender email address");
        let timeout = cfg.email_client.timeout();
        let email_client = EmailClient::new(
            cfg.email_client.base_url,
            sender,
            cfg.email_client.auth_token,
            timeout,
        );

        let listener = TcpListener::bind(cfg.service.host).expect("failed to bind port");
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, pg_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

fn run(
    listener: TcpListener,
    pg_pool: PgPool,
    email_client: EmailClient,
) -> std::io::Result<Server> {
    let database = web::Data::new(pg_pool);

    let srv = HttpServer::new(move || {
        actix_web::App::new()
            .wrap(TracingLogger::default())
            .route("/healthz", web::get().to(healthz))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(database.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .client_disconnect_timeout(Duration::from_secs(10))
    .shutdown_timeout(10)
    .run();

    Ok(srv)
}
