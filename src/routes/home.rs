use std::io;

use actix_files::NamedFile;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use askama::Template;
use sqlx::PgPool;

use crate::error::KBResult;
use crate::replacements::markdown2html;
use crate::routes::pages::{PageInfo, PageSummary};

#[derive(Template)]
#[template(path = "home.html", escape = "none")]
struct HomePage {
    recently_edited: Vec<PageSummary>,
}

impl HomePage {
    fn render(recently_edited: Vec<PageSummary>) -> KBResult<String> {
        Ok(HomePage {
            recently_edited,
        }
            .render()
            .context("Could not render home page.")?)
    }
}

pub async fn home(pool: web::Data<PgPool>) -> KBResult<HttpResponse> {
    let recently_edited = retrieve_recently_edited(&pool)
        .await
        .context("Failed to fetch recently edited pages.")?
        .into_iter()
        .map(|page| PageSummary {
            url: format!("pages/{}", page.title),
            title: page.title,
            updated_at: page.updated_at.format("%Y-%m-%d").to_string(),
            summary: page.summary.and_then(
                |summary| markdown2html(&summary, "pages/").ok()),
        }).collect();
    Ok(HttpResponse::Ok()
        .body(HomePage::render(recently_edited)?))
}

async fn retrieve_recently_edited(
    pool: &PgPool,
) -> Result<Vec<PageInfo>, sqlx::Error> {
    sqlx::query_as!(PageInfo, r#"
    SELECT title, summary, updated_at
    FROM page
    ORDER BY updated_at DESC
    LIMIT 5;
    "#)
        .fetch_all(pool)
        .await
}


pub async fn favicon() -> io::Result<NamedFile> {
    NamedFile::open_async("public/assets/favicon.ico").await
}