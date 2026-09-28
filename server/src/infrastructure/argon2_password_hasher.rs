use crate::{application::services::password_hasher_service::PasswordHasherService, error::ApiError};
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
    async fn hash(&self, raw_password: String) -> Result<String, ApiError> {
        let _permit = HASH_SEMAPHORE
            .acquire()
            .await
            .map_err(|_| ApiError::Unexpected)?;
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(raw_password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|_| ApiError::Unexpected)
        })
        .await
        .map_err(|_| ApiError::Unexpected)?
    }

    async fn verify(&self, raw_password: String, hash: String) -> Result<bool, ApiError> {
        let _permit = HASH_SEMAPHORE
            .acquire()
            .await
            .map_err(|_| ApiError::Unexpected)?;
        tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash).map_err(|_| ApiError::Unexpected)?;
            Ok(Argon2::default()
                .verify_password(raw_password.as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|_| ApiError::Unexpected)?
    }
}
