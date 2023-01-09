use actix_web::{HttpResponse, web};
use actix_web::error::InternalError;
use actix_web_flash_messages::FlashMessage;
use secrecy::Secret;
use sqlx::PgPool;

use crate::authentication::{Credentials, validate_credentials};
use crate::error::KBError;
use crate::session_state::TypedSession;
use crate::utils::see_other;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<KBError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            session.renew();
            session.insert_user_id(user_id)
                .map_err(|e| login_redirect(KBError::UnexpectedError(e.into())))?;
            Ok(see_other("/admin"))
        }
        Err(e) => {
            Err(login_redirect(e))
        }
    }
}

fn login_redirect(e: KBError) -> InternalError<KBError> {
    FlashMessage::error(e.to_string()).send();
    let response =  see_other("/login");
    InternalError::from_response(e, response)
}