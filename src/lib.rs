#![allow(dead_code)]

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn subscriptions(_form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(tcp_listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
    })
    .listen(tcp_listener)
    .unwrap()
    .run();

    Ok(server)
}
