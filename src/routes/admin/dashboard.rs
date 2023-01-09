use actix_web::{HttpResponse, web};
use actix_web::http::header::LOCATION;
use anyhow::Context;
use askama::Template;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::KBResult;
use crate::session_state::TypedSession;
use crate::utils::e500;

#[derive(Template)]
#[template(path = "admin/dashboard.html", escape = "none")]
struct AdminDashboard<'a> {
    username: &'a str,
}

impl AdminDashboard<'_> {
    fn render(username: &str) -> KBResult<String> {
        Ok(AdminDashboard { username }
            .render().context("Could not render dashboard view ")?)
    }
}


pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session
        .get_user_id()
        .map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };
    // TODO: render template
    Ok(HttpResponse::Ok().body(
        AdminDashboard::render(&username)?
    ))
}

async fn get_username(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(r#"
        SELECT username
        FROM users
        WHERE user_id = $1
    "#, user_id)
        .fetch_one(pool)
        .await
        .context("Failed to retrieve username")?;
    Ok(row.username)
}