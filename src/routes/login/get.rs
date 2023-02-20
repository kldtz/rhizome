use actix_web::{HttpResponse, web};
use actix_web_flash_messages::IncomingFlashMessages;
use anyhow::Context;
use askama::Template;
use serde::Deserialize;

use crate::error::KBResult;
use crate::utils::collect_messages_as_html;

#[derive(Template)]
#[template(path = "login.html", escape = "none")]
struct LoginPage<'a> {
    error_message: &'a str,
    redirect: &'a Option<String>,
}

#[derive(Deserialize)]
pub struct Info {
    pub redirect: Option<String>,
}

pub async fn login_form(flash_messages: IncomingFlashMessages, info: web::Query<Info>) -> KBResult<HttpResponse> {
    let error_html = collect_messages_as_html(flash_messages);
    Ok(HttpResponse::Ok().body(LoginPage { error_message: &error_html, redirect: &info.redirect }
        .render()
        .context("Could not render login page.")?
    ))
}