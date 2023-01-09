use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use anyhow::Context;
use askama::Template;

use crate::error::KBResult;
use crate::utils::collect_messages_as_html;

#[derive(Template)]
#[template(path = "admin/change_password.html", escape = "none")]
struct ChangePasswordPage<'a> {
    error_message: &'a str,
}

pub async fn change_password_form(flash_messages: IncomingFlashMessages) -> KBResult<HttpResponse> {
    let error_message = collect_messages_as_html(flash_messages);
    Ok(HttpResponse::Ok().body(
        ChangePasswordPage { error_message: &error_message }
            .render()
            .context("Could not render change password page.")?
    ))
}