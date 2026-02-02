use crate::helpers::{spawn_app, teardown};

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
    teardown(test_app).await;
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

    teardown(test_app).await;
}
