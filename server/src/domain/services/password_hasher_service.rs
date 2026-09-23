use async_trait::async_trait;

use crate::error::ApiError;

#[async_trait]
pub trait PasswordHasherService: Send + Sync {
    async fn hash(&self, raw_password: String) -> Result<String, ApiError>;
    async fn verify(&self, raw_password: String, hash: String) -> Result<bool, ApiError>;
}
