use actix_web::{HttpResponse, ResponseError};
use actix_web::http::{StatusCode, header::ContentType};

pub type KBResult<T> = std::result::Result<T, KBError>;

#[derive(thiserror::Error, Debug)]
pub enum KBError {
    #[error("{0}")]
    LoginError(String),
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for KBError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}