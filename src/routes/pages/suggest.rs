use actix_web::web;
use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use anyhow::Context;
use askama::Template;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::KBResult;

#[derive(Template)]
#[template(path = "pages/suggestions.html", escape = "none")]
struct Suggestions {
    suggestions: Vec<Suggestion>,
}

impl Suggestions {
    fn render(ids: Vec<Suggestion>) -> KBResult<String> {
        Ok(Suggestions { suggestions: ids }.render()
            .context("Could not render suggestions")?)
    }
}

#[derive(Deserialize)]
pub struct Info {
    pub value: String,
}

pub async fn suggest_page_title(pool: web::Data<PgPool>, form: web::Query<Info>) -> KBResult<HttpResponse> {
    let pattern = format!("{}%", form.value);
    let pages = select_pages(pool.get_ref(), &pattern)
        .await
        .context("Failed to fetch suggestions.")?
        .into_iter()
        .collect();
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(Suggestions::render(pages)?))
}

struct Suggestion {
    id: i32,
    title: String,
}

async fn select_pages(
    pool: &PgPool,
    pattern: &str,
) -> Result<Vec<Suggestion>, sqlx::Error> {
    sqlx::query_as!(Suggestion, r#"
    SELECT id, title
    FROM page
    WHERE title ILIKE $1
    "#, pattern)
        .fetch_all(pool)
        .await
}