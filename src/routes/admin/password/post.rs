use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;
use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};
use anyhow::Context;
use crate::authentication;
use crate::authentication::{Credentials, validate_credentials};
use crate::error::KBError;

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session.get_user_id().map_err(e500)?;
    if user_id.is_none() {
        return Ok(see_other("/login"));
    }
    let user_id = user_id.unwrap();
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match."
        ).send();
        return Ok(see_other("/admin/password"));
    }
    // TODO: optionally add some validations

    let username = get_username(&pool, user_id).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            KBError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            _ => Err(e500(e).into())
        }
    }
    authentication::change_password(user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::info("Your password has been changed.").send();
    Ok(see_other("/admin/password"))
}

async fn get_username(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(r#"
    SELECT username
    FROM users
    WHERE user_id = $1
    "#, user_id)
        .fetch_one(pool)
        .await
        .context("Failed retrieve username.")?;
    Ok(row.username)
}