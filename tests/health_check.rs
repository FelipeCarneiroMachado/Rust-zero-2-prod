use reqwest;
use secrecy::SecretBox;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use std::sync::LazyLock;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup::run;
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

struct TestApp {
    address: String,
    db_pool: PgPool,
    test_db_settings: DatabaseSettings,
}

async fn get_maintenance_db(config: &DatabaseSettings) -> PgConnection {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretBox::new(Box::new("password".to_string())),
        port: config.port.clone(),
        host: config.host.clone(),
    };

    PgConnection::connect(&maintenance_settings.connection_string())
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

    let pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate");
    pool
}

async fn teardown(config: DatabaseSettings) {
    let mut connection = get_maintenance_db(&config).await;
    connection
        .execute(format!(r#"DROP DATABASE "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to drop the database");
}
async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind socket at random port");
    let port = listener.local_addr().unwrap().port();
    let mut config = get_configuration().expect("Failed to get configuration");

    // Test DB
    config.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection = setup_db(&config.database).await;
    let server = run(listener, connection.clone()).expect("Failed to start server");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection,
        test_db_settings: config.database,
    }
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let address = &test_app.address;
    // let config = get_configuration().expect("Failed to get configuration");

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Can't get response");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    test_app.db_pool.close().await;
    teardown(test_app.test_db_settings).await;
}

#[tokio::test]
async fn post_returns_200() {
    //setup
    let test_app = spawn_app().await;
    let address = &test_app.address;
    let client = reqwest::Client::new();
    // let config = get_configuration().expect("Failed to read configuration");
    let connection = &test_app.db_pool;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // act
    let response = client
        .post(format!("{}/subscribe", &address))
        .header("Content-Length", body.len())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Can't get response");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(connection)
        .await
        .expect("Failed to fetch saved subscriptions");

    // let saved : (String, String) = (saved.get("email"), saved.get("name"));
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    assert_eq!(response.status().as_u16(), 200u16);
    connection.close().await;
    teardown(test_app.test_db_settings).await;
}

#[tokio::test]
async fn post_returns_400() {
    //setup
    let test_app = spawn_app().await;
    let address = &test_app.address;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, cause) in test_cases {
        let response = client
            .post(format!("{}/subscribe", &address))
            .header("Content-Length", invalid_body.len())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Can't get response");

        assert_eq!(
            400u16,
            response.status().as_u16(),
            "API didn't fail with {cause}"
        );
    }
    test_app.db_pool.close().await;
    teardown(test_app.test_db_settings).await;
}
