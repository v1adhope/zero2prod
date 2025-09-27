use crate::{
    email_client::EmailClient,
    routes::{healthz, subscribe},
};
use actix_web::{App, HttpServer, dev::Server, web};
use sqlx::PgPool;
use std::{net::TcpListener, time::Duration};
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    database: PgPool,
    email_client: EmailClient,
) -> std::io::Result<Server> {
    let database = web::Data::new(database);

    let srv = HttpServer::new(move || {
        App::new()
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
