use std::fmt::Write;

use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;

// Return an opaque 500 while preserving the error root's cause for logging.
pub fn e500<T>(e: T) -> actix_web::Error
    where
        T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

// Return a 400 with the user-representation of the validation error as body.
// The error root cause is preserved for logging purposes.
pub fn e400<T: std::fmt::Debug + std::fmt::Display>(e: T) -> actix_web::Error
    where
        T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorBadRequest(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

pub fn collect_messages_as_html(flash_messages: IncomingFlashMessages) -> String {
    let mut message_html = String::new();
    for m in flash_messages.iter() {
        writeln!(message_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    message_html
}