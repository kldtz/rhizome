use actix_web::{FromRequest, HttpMessage};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::InternalError;
use actix_web::http::Method;
use actix_web_lab::middleware::Next;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use uuid::Uuid;

use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};

#[derive(Copy, Clone, Debug)]
pub struct UserId(Uuid);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub async fn reject_anonymous_users(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = {
        let (http_request, payload) = req.parts_mut();
        TypedSession::from_request(http_request, payload).await
    }?;

    match session.get_user_id().map_err(e500)? {
        Some(user_id) => {
            req.extensions_mut().insert(UserId(user_id));
            next.call(req).await
        }
        None => {
            let location = match req.method() {
                // Only redirect GET requests
                &Method::GET => {
                    let uri = req.uri().to_string();
                    let encoded_uri = utf8_percent_encode(
                        &uri, NON_ALPHANUMERIC);
                    format!("/login?redirect={}", encoded_uri)
                }
                _ => String::from("/login")
            };
            let response = see_other(&location);
            let e = anyhow::anyhow!("The user has not logged in.");
            Err(InternalError::from_response(e, response).into())
        }
    }
}