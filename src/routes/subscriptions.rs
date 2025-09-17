use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use sqlx::{PgPool, types::chrono::Utc};
use tracing::Level;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubscribeForm {
    name: String,
    email: String,
}

#[tracing::instrument(
    level = Level::DEBUG,
    name = "Adding a new subscriber",
    skip_all,
    fields(sub_email = %form.email, sub_name = %form.name),
)]
pub async fn subscribe(
    form: web::Form<SubscribeForm>,
    database: web::Data<PgPool>,
) -> impl Responder {
    match insert_subscriber(&form, &database).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("failed to execute query: {e:?}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    level = Level::DEBUG,
    name = "saving new subscriber details in the database",
    skip_all,
)]
pub async fn insert_subscriber(form: &SubscribeForm, database: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions(subscription_id, email, name, created_at)
        values($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(database)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {e:?}");
        e
    })?;
    Ok(())
}
