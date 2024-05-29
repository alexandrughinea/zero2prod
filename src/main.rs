use zero2prod::{configuration, startup, telemetry};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = configuration::get_configuration().expect("Failed to read configuration.");

    let application = startup::Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
