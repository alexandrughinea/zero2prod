use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use zero2prod::configuration::{get_configuration, get_subscriber, init_subscriber};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed. // All other invocations will instead skip execution.
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database().await;

    let server =
        run(listener, connection_pool.clone(), email_client).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database() -> PgPool {
    let mut configuration = get_configuration().expect("Failed to read configuration.");

    // For each new test run create a new random db name
    configuration.database.database_name = Uuid::new_v4().to_string();

    // Create database
    let mut connection = PgConnection::connect_with(&configuration.database.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(
            format!(
                r#"CREATE DATABASE "{}";"#,
                configuration.database.database_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect_with(configuration.database.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let request_url = format!("{}/health_check", &app.address);

    let response = client
        .get(&request_url)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0))
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration");
    let mut connection = PgConnection::connect_with(&configuration.database.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();
    let request_url = format!("{}/subscriptions", &app.address);
    let request_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&request_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(request_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let request_url = format!("{}/subscriptions", &app.address);

    let test_cases = vec![
        ("name=le%20", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, _error_message) in test_cases {
        let response = client
            .post(&request_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(response.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
