use crate::application::{
    error::AppError, services::password_hasher_service::PasswordHasherService,
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Semaphore;

lazy_static::lazy_static! {
    static ref HASH_SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(
        num_cpus::get().saturating_sub(1).max(1)
    ));
}

pub struct Argon2PasswordHasher;
#[async_trait]
impl PasswordHasherService for Argon2PasswordHasher {
    async fn hash(&self, raw_password: String) -> Result<String, AppError> {
        let _permit = HASH_SEMAPHORE.acquire().await.map_err(|e| {
            tracing::error!(error = ?e, "Failed to acquire hash semaphore permit");
            AppError::internal("Internal processing error")
        })?;

        let hashed = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(raw_password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| {
                    // Log the detailed crypto error internally
                    tracing::error!(error = ?e, "Argon2 password hashing failed");
                    // Return a clean error message to the app layer
                    "Password hashing failed"
                })
        })
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "Blocking task panicked or failed during hashing");
            AppError::internal("Internal processing error")
        })?;

        // Map the string slice error from the thread to an AppError
        hashed.map_err(|e| AppError::internal(e))
    }

    async fn verify(&self, raw_password: String, hash: String) -> Result<bool, AppError> {
        let _permit = HASH_SEMAPHORE.acquire().await.map_err(|e| {
            tracing::error!(error = ?e, "Failed to acquire hash semaphore permit");
            AppError::internal("Internal processing error")
        })?;

        let verified = tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash).map_err(|e| {
                tracing::error!(error = ?e, "Failed to parse stored password hash");
                "Invalid password hash format"
            })?;

            Ok(Argon2::default()
                .verify_password(raw_password.as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "Blocking task panicked or failed during password verification");
            AppError::internal("Internal processing error")
        })?;

        verified.map_err(|e: &str| AppError::internal(e))
    }
}
