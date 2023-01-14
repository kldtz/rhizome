use actix_web::{HttpResponse};
use actix_web::web;
use anyhow::Context;
use chrono::Utc;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};

use crate::error::KBResult;
use crate::page::{Link, Page};
use crate::utils::see_other;

#[derive(Deserialize)]
pub struct Info {
    pub value: String,
}

pub async fn create_page(pool: web::Data<PgPool>, form: web::Form<Info>) -> KBResult<HttpResponse> {
    let new_id = form.value.trim();
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a postgres connection from the pool.")?;
    insert_new_page(&mut transaction, new_id)
        .await
        .context("Failed to insert new page.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store new page.")?;
    // TODO: display error on current page if this fails
    Ok(see_other(&format!("/pages/{}/edit", new_id)))
}

async fn insert_new_page(transaction: &mut Transaction<'_, Postgres>, title: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"
    INSERT INTO page (title, summary, content, created_at)
    VALUES ($1, NULL, $2, $3)
    "#, title, "", Utc::now())
        .execute(transaction)
        .await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct EditForm {
    pub markdown: String,
}

pub async fn save_page(
    pool: web::Data<PgPool>,
    id: web::Path<String>,
    form: web::Form<EditForm>,
) -> KBResult<HttpResponse> {
    let title = id.into_inner();
    let page = Page::unique(Page::from(&form.markdown));
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a postgres connection from the pool.")?;
    update_page(&mut transaction, &page, &title)
        .await
        .context("Failed to update page.")?;
    delete_links(&mut transaction, &title)
        .await
        .context("Failed to delete old links.")?;
    if !page.links.is_empty() {
        write_links(&mut transaction, &title, page.links)
            .await
            .context("Failed to write links.")?;
    }
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store new page.")?;
    Ok(HttpResponse::SeeOther()
        .append_header(("Location", format!("/pages/{}", title)))
        .finish())
}

async fn update_page(
    transaction: &mut Transaction<'_, Postgres>,
    page: &Page,
    title: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"
    UPDATE page
    SET content = $1, summary = $2, updated_at = $3
    WHERE title = $4;
    "#, page.text, page.summary, Utc::now(), title)
        .execute(transaction)
        .await?;
    Ok(())
}

async fn delete_links(
    transaction: &mut Transaction<'_, Postgres>,
    source: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"
    DELETE FROM link
    WHERE link.source = (
        SELECT id
        FROM page
        WHERE title = $1
    );
    "#, source)
        .execute(transaction)
        .await?;
    Ok(())
}

async fn write_links(
    transaction: &mut Transaction<'_, Postgres>,
    source: &str,
    links: Vec<Link>,
) -> Result<(), sqlx::Error> {
    let (targets, directions): (Vec<String>, Vec<bool>) = links.into_iter().map(|l| (l.target, l.bidirectional)).unzip();
    sqlx::query!(r#"
    INSERT INTO link (source, target, bidirectional)
    SELECT source_page.id, target_page.id, new_link.bidirectional
    FROM UNNEST($1::text[], $2::bool[]) AS new_link (target, bidirectional)
    INNER JOIN page source_page ON source_page.title = $3
    INNER JOIN page target_page ON target_page.title = new_link.target;
    "#, &targets, &directions, source)
        .execute(transaction)
        .await?;
    Ok(())
}