use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use argon2::password_hash::SaltString;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use tokio::task::spawn_blocking;

use crate::error::{KBError, KBResult};

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(r#"
    SELECT user_id, password_hash
    FROM users
    WHERE username = $1
   "#, username)
        .fetch_optional(pool)
        .await
        .context("Failed to retrieve stored credentials.")?
        .map(|row| (row.user_id, Secret::new(row.password_hash)));
    Ok(row)
}

pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> KBResult<uuid::Uuid> {
    let mut user_id = None;
    // Dummy hash that is validated in failure case for equal response time
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    // Retrieve expected password hash from db
    if let Some((stored_user_id, stored_password_hash)) =
    get_stored_credentials(&credentials.username, pool).await? {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    // Verify if password matches stored hash in separate blocking thread (bc expensive)
    spawn_blocking(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
        .await
        .context("Failed to spawn blocking task.")?
        .await?;

    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username."))
        .map_err(KBError::InvalidCredentials)
}

pub async fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> KBResult<()> {
    // Parse PHC string: $<id>[$v=<version>][$<param>=<value>(,<param>=<value>)*][$<salt>[$<hash>]]
    // see https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md for details
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    // Compute hash for password candidate and compare
    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(KBError::InvalidCredentials)
}

pub async fn change_password(
    user_id: uuid::Uuid,
    password: Secret<String>,
    pool: &PgPool,
) -> Result<(), anyhow::Error> {
    let password_hash = spawn_blocking(
        move || compute_password_hash(password)
    )
        .await?
        .context("Failed to hash password")?;

    sqlx::query!(r#"
    UPDATE users
    SET password_hash = $1
    WHERE user_id = $2
    "#, password_hash.expose_secret(), user_id)
        .execute(pool)
        .await
        .context("Failed to change user password in the database.")?;
    Ok(())
}

fn compute_password_hash(
    password: Secret<String>
) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string();
    Ok(Secret::new(password_hash))
}