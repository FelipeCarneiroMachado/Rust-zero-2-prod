use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{health_check, index, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    pub port: u16,
    pub server: Server,
}

pub fn get_connection_pool(config: &Settings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(config.database.connection_options())
}

pub fn get_server_address(config: &Settings) -> String {
    format!("{}:{}", &config.application.host, &config.application.port)
}

pub fn build(config: Settings) -> Result<Application, std::io::Error> {
    let email_client = EmailClient::new(
        reqwest::Url::parse(config.email_client.base_url.as_str())
            .expect("Failed to parse base url from config"),
        config
            .email_client
            .sender()
            .expect("Failed to parse sender email"),
        config.email_client.auth_token,
        std::time::Duration::from_millis(config.email_client.timeout_ms),
    );
    let address = format!("{}:{}", &config.application.host, &config.application.port);

    let listener = TcpListener::bind(address).expect("Failed to bind random port");

    let connection = PgPoolOptions::new().connect_lazy_with(config.database.connection_options());

    Ok(Application {
        port: listener.local_addr()?.port(),
        server: run(listener, connection, email_client)?,
    })
}

pub fn run(
    tcp_listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .route("/", web::get().to(index))
            .app_data(connection.clone())
            .app_data(web::Data::clone(&email_client))
    })
    .listen(tcp_listener)?
    .run();
    Ok(server)
}
