use crate::routes::{health_check, subscriptions};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgConnection;
use std::net::TcpListener;

pub fn run(tcp_listener: TcpListener, connection: PgConnection) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer (ARC pointer)
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            // Register the connection as part of the application state .app_data(connection)
            .app_data(connection.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
    })
    .listen(tcp_listener)
    .unwrap()
    .run();

    Ok(server)
}
