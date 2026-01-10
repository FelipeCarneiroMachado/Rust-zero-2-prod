use actix_web::web::Form;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::types::chrono;
use sqlx::PgPool;
use uuid::Uuid;


#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct RegisterData {
    email: String,
    name: String,
}
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe_user(form: Form<RegisterData>, connection: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();

    tracing::info!("Request_id[{request_id}] - Saving new subscriber to database: {:?}", form);
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
        .execute(connection.get_ref())
        .await {
        Ok(_) => {
            tracing::info!("Request_id[{request_id}] - Subscriber info registered successfully");
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("Request_id[{request_id}] - Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}