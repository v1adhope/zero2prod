use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use sqlx::{PgPool, types::chrono::Utc};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubscribeForm {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: web::Form<SubscribeForm>,
    database: web::Data<PgPool>,
) -> impl Responder {
    match sqlx::query!(
        r#"
        insert into subscriptions(subscription_id, email, name, created_at)
        values($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(database.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("failed to execute query: {}", e);
            HttpResponse::InternalServerError()
        }
    }
}
