use std::net::TcpListener;

use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::{App, HttpServer, web};
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_lab::middleware::from_fn;
use secrecy::ExposeSecret;
use secrecy::Secret;
use sqlx::PgPool;

use crate::authentication::reject_anonymous_users;
use crate::configuration::Settings;
use crate::routes::{admin_dashboard, change_password, change_password_form, create_page, edit_page, favicon, health_check, home, log_out, login, login_form, read_page, save_page, search_page, suggest_page_title};

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

pub async fn build(configuration: Settings) -> Result<Server, anyhow::Error> {
    let connection_pool = PgPool::connect_lazy(&configuration.database.connection_string())
        .expect("Failed to connect to Postgres.");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(
        listener,
        connection_pool,
        configuration.application.hmac_secret,
        configuration.redis_uri,
    ).await
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(redis_store.clone(), secret_key.clone()))
            // Public endpoints
            .route("/health_check", web::get().to(health_check))
            .service(actix_files::Files::new("/static", "public"))
            .route("/favicon.ico", web::get().to(favicon))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            // Private endpoints that require login
            .service(web::scope("/")
                .wrap(from_fn(reject_anonymous_users))
                .route("", web::get().to(home))
                .route("/suggest", web::get().to(suggest_page_title))
            )
            .service(web::scope("/pages")
                .wrap(from_fn(reject_anonymous_users))
                .route("", web::get().to(search_page))
                .route("", web::post().to(create_page))
                .route("/{id}", web::get().to(read_page))
                .route("/{id}/edit", web::get().to(edit_page))
                .route("/{id}/edit", web::post().to(save_page))
            )
            .service(web::scope("/admin")
                .wrap(from_fn(reject_anonymous_users))
                .route("", web::get().to(admin_dashboard))
                .route("/password", web::get().to(change_password_form))
                .route("/password", web::post().to(change_password))
                .route("/logout", web::post().to(log_out))
            )
            .app_data(db_pool.clone())
            .app_data(Data::new(HmacSecret(hmac_secret.clone())))
    })
        .listen(listener)?
        .run();
    // No .await here!
    Ok(server)
}