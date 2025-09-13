use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use std::time::Duration;

async fn healthz() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/healthz", web::get().to(healthz)))
        .bind("127.0.0.1:8080")?
        .client_disconnect_timeout(Duration::from_secs(10))
        .shutdown_timeout(10)
        .run()
        .await
}
