use actix_web::{App, HttpResponse, HttpServer, Responder, dev::Server, web};
use std::{net::TcpListener, time::Duration};

async fn healthz() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let srv = HttpServer::new(|| App::new().route("/healthz", web::get().to(healthz)))
        .listen(listener)?
        .client_disconnect_timeout(Duration::from_secs(10))
        .shutdown_timeout(10)
        .run();

    Ok(srv)
}
