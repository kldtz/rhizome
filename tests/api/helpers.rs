use std::net::TcpListener;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use argon2::password_hash::SaltString;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use rhizome::configuration::{DatabaseSettings, get_configuration};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn create_page(&self, page_title: &str) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/pages", &self.address))
            .form(&[("value", page_title)])
            .send()
            .await
            .expect("Failed to execute page creation request.")
    }

    pub async fn get_login(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }


    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where Body: serde::Serialize {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_change_password(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/password", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response
    where Body: serde::Serialize, {
        self.api_client
            .post(&format!("{}/admin/password", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/admin/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

// Launch our application in the background
pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = rhizome::startup::run(
            listener,
            connection_pool.clone(),
            configuration.application.static_dir,
            configuration.application.hmac_secret,
            configuration.redis_uri,
        ).await.expect("Failed to bind address");

    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address,
        db_pool: connection_pool,
        test_user: TestUser::new(),
        api_client,
    };

    test_app.test_user.store(&test_app.db_pool).await;

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub struct TestUser {
    user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn new() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    pub async fn login(&self, app: &TestApp) -> reqwest::Response {
        app.post_login(&serde_json::json!({
            "username": &self.username,
            "password": &self.password,
        }))
            .await
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap()
        )
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        sqlx::query!(r#"
            INSERT INTO users (user_id, username, password_hash)
            VALUES ($1, $2, $3);
        "#,
            self.user_id,
            self.username,
            password_hash,
        )
            .execute(pool)
            .await
            .expect("Failed to store test user.");
    }
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}