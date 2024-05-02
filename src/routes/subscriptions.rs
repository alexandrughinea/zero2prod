use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;

use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection_pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscriptions(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_subscriber(&connection_pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, connection_pool)
)]
pub async fn insert_subscriber(
    connection_pool: &PgPool,
    form: &FormData,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection_pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(error) => {
            tracing::error!("Failed to execute query: {:?}", error);
            Err(error)
        }
    }
}
