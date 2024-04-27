use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

use crate::routes::{health_check, subscriptions};

pub fn run(tcp_listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the `connection_pool` in a smart pointer (ARC pointer)
    let connection_pool = web::Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            // Register the connection as part of the application state .app_data(connection)
            .app_data(connection_pool.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
    })
    .listen(tcp_listener)
    .unwrap()
    .run();

    Ok(server)
}
