use actix_web::web;
use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use anyhow::Context;
use askama::Template;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::KBResult;
use crate::replacements::markdown2html;
use crate::routes::pages::PageSummary;

#[derive(Template)]
#[template(path = "pages/results.html", escape = "none")]
struct SearchResults<'a> {
    query: &'a str,
    results: Vec<PageSummary>,
}

impl SearchResults<'_> {
    fn render(query: &str, results: Vec<PageSummary>) -> KBResult<String> {
        Ok(SearchResults {
            query,
            results,
        }
            .render()
            .with_context(|| format!("Could not render search results for query '{}'", query))?)
    }
}

#[derive(Deserialize)]
pub struct Info {
    pub value: String,
}

pub async fn search_page(pool: web::Data<PgPool>, info: web::Query<Info>) -> KBResult<HttpResponse> {
    let pattern = &info.value;
    let results = select_pages(pool.get_ref(), pattern)
        .await
        .context("Failed to fetch search results.")?
        .into_iter()
        .map(|page| PageSummary {
            url: format!("pages/{}", page.title),
            title: page.title,
            updated_at: page.updated_at.format("%Y-%m-%d").to_string(),
            summary: page.summary
                .and_then(|summary| markdown2html(&summary,
                                                  "pages/").ok()),
        })
        .collect();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(SearchResults::render(&info.value, results)?))
}

struct PageView {
    title: String,
    summary: Option<String>,
    updated_at: DateTime<Utc>,
}

async fn select_pages(
    pool: &PgPool,
    pattern: &str,
) -> Result<Vec<PageView>, sqlx::Error> {
    sqlx::query_as!(PageView, r#"
    SELECT title, summary, updated_at
    FROM page
    WHERE content ~* $1 OR title ~* $1
    "#, pattern)
        .fetch_all(pool)
        .await
}
