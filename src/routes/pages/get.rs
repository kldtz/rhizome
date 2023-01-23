use actix_web::web;
use actix_web::HttpResponse;
use anyhow::Context;
use askama::Template;
use sqlx::PgPool;

use crate::error::KBResult;
use crate::replacements::markdown2html;
use crate::routes::pages::{PageInfo, PageSummary};

#[derive(Template)]
#[template(path = "pages/read.html", escape = "none")]
struct ReadPage<'a> {
    title: &'a str,
    id: &'a str,
    content: &'a str,
    backlinks: Vec<PageSummary>,
}

impl ReadPage<'_> {
    fn render(id: &str, title: &str, content: &str, backlinks: Vec<PageInfo>) -> KBResult<String> {
        Ok(ReadPage {
            title,
            id,
            content,
            backlinks: backlinks.into_iter().map(|bl| PageSummary {
                url: bl.title.to_string(),
                title: bl.title,
                updated_at: bl.updated_at.format("%Y-%m-%d").to_string(),
                summary: bl.summary.and_then(
                    |summary| markdown2html(&summary, "").ok()),
            }).collect(),
        }
            .render().with_context(|| format!("Could not render read view for pages {}", title))?)
    }
}

pub async fn read_page(pool: web::Data<PgPool>, path: web::Path<String>) -> KBResult<HttpResponse> {
    let title = path.into_inner();
    let page = select_page(pool.get_ref(), &title)
        .await
        .context("Failed to fetch page.")?;
    let backlinks = retrieve_backlinks(pool.get_ref(), page.id)
        .await
        .context("Failed to fetch backlinks")?;
    let html_content = markdown2html(&page.content, "")?;
    Ok(HttpResponse::Ok().body(ReadPage::render(&title, &title, &html_content, backlinks)?))
}

struct Page {
    id: i32,
    content: String,
}

async fn select_page(
    pool: &PgPool,
    title: &str,
) -> Result<Page, sqlx::Error> {
    sqlx::query_as!(Page, r#"
    SELECT id, content
    FROM page
    WHERE title = $1
    "#, title
    )
        .fetch_one(pool)
        .await
}

async fn retrieve_backlinks(
    pool: &PgPool,
    target: i32,
) -> Result<Vec<PageInfo>, sqlx::Error> {
    sqlx::query_as!(PageInfo, r#"
    SELECT page.title AS title, page.summary AS summary, page.updated_at AS updated_at
    FROM link
    INNER JOIN page ON page.id = link.source
    WHERE link.target = $1
    ORDER BY page.title
    "#, target)
        .fetch_all(pool)
        .await
}

#[derive(Template)]
#[template(path = "pages/edit.html", escape = "none")]
struct EditPage<'a> {
    id: &'a str,
    title: &'a str,
    content: &'a str,
}

impl EditPage<'_> {
    fn render(id: &str, title: &str, content: &str) -> KBResult<String> {
        Ok(EditPage {
            id,
            title,
            // Backticks need escaping because this goes into a JS template literal
            content: &content.replace('`', r"\`"),
        }
            .render().with_context(|| format!("Could not render edit view for pages {}", title))?)
    }
}

pub async fn edit_page(pool: web::Data<PgPool>, path: web::Path<String>) -> KBResult<HttpResponse> {
    let title = path.into_inner();
    let page = select_page(pool.get_ref(), &title)
        .await
        .context("Failed to fetch page.")?;
    Ok(HttpResponse::Ok().body(EditPage::render(&title, &title, &page.content)?))
}