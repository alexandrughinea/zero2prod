use crate::email_client::EmailClient;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscriptions};

pub fn run(
    tcp_listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool); // Wrap the `connection_pool` in a smart pointer (ARC pointer)
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default()) // Register the connection as part of the application state .app_data(connection)
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
    })
    .listen(tcp_listener)
    .unwrap()
    .run();

    Ok(server)
}
