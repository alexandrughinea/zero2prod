use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::{get_configuration, get_subscriber, init_subscriber};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    // Build an `EmailClient` using `configuration`
    let timeout = configuration.email_client.timeout();
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    // We are falling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set.
    let connection_pool = PgPool::connect_lazy_with(configuration.database.with_db());
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind to random port.");

    run(listener, connection_pool, email_client).unwrap().await
}
