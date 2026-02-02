use actix_web::dev::ServerHandle;
use secrecy::SecretBox;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::build;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let sink = std::env::var("TEST_LOG").is_ok();

    if sink {
        let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("zero2prod".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub test_db_settings: DatabaseSettings,
    pub server: ServerHandle,
}

async fn get_maintenance_db(config: &DatabaseSettings) -> PgConnection {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretBox::new(Box::new("password".to_string())),
        port: config.port.clone(),
        host: config.host.clone(),
        require_ssl: false,
    };

    PgConnection::connect_with(&maintenance_settings.connection_options())
        .await
        .expect("failed to connect to maintenance database")
}

async fn setup_db(config: &DatabaseSettings) -> PgPool {
    // Connect to postgres
    let mut connection = get_maintenance_db(config).await;
    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPoolOptions::new()
        .connect_with(config.connection_options())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate");
    pool
}

pub async fn teardown(app: TestApp) {
    app.db_pool.close().await;

    app.server.stop(true).await;

    let mut connection = get_maintenance_db(&app.test_db_settings).await;
    connection
        .execute(format!(r#"DROP DATABASE "{}""#, app.test_db_settings.database_name).as_str())
        .await
        .expect("Failed to drop the database");
}
pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let config = {
        let mut config = get_configuration().expect("Failed to get configuration");
        config.database.database_name = format!("test_{}", uuid::Uuid::new_v4().to_string());

        config.application.port = 0;
        config
    };

    // Test DB
    let connection = setup_db(&config.database).await;
    let application = build(config.clone()).expect("Failed to build server");
    let handle = application.server.handle();
    let _ = tokio::spawn(application.server);
    TestApp {
        address: format!("http://127.0.0.1:{}", application.port),
        db_pool: connection,
        test_db_settings: config.database,
        server: handle,
    }
}
