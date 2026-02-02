use zero2prod::configuration::get_configuration;
use zero2prod::startup::build;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Logging setup
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Socket and db setup
    let config = get_configuration().expect("Failed to read configuration");

    println!("STARTING SERVER WITH CONFIG {:?}", config);

    let application = build(config)?;
    application.server.await?;
    Ok(())
}
