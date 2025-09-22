use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use sqlx::{PgPool, types::chrono::Utc};
use tracing::Level;
use uuid::Uuid;

use crate::domain::{Email, NewSubscriber, SubscriberName};

#[derive(Deserialize)]
pub struct SubscribeForm {
    pub name: String,
    pub email: String,
}

impl TryFrom<SubscribeForm> for NewSubscriber {
    type Error = String;

    fn try_from(value: SubscribeForm) -> Result<Self, Self::Error> {
        let email = Email::parse(value.email)?;
        let name = SubscriberName::parse(value.name)?;
        Ok(Self {
            email: email,
            name: name,
        })
    }
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
    let subscriber = match form.0.try_into() {
        Ok(m) => m,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&subscriber, &database).await {
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
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    database: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into subscriptions(subscription_id, email, name, created_at)
        values($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
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
