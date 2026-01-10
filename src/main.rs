use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Logging setup
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Socket and db setup
    let config = get_configuration().expect("Failed to read configuration");

    let address = format!("{}:{}", &config.application.host, &config.application.port);

    let listener = TcpListener::bind(address).expect("Failed to bind random port");

    let connection = PgPool::connect_lazy(&config.database.connection_string())
        .expect("Failed to connect to database");

    run(listener, connection)?.await
}
