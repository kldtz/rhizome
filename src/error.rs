use actix_web::http::StatusCode;
use actix_web::ResponseError;

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
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}