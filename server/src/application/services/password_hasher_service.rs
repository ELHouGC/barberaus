use async_trait::async_trait;

use crate::application::error::AppError;



#[async_trait]
pub trait PasswordHasherService: Send + Sync {
    async fn hash(&self, raw_password: String) -> Result<String, AppError>;
    async fn verify(&self, raw_password: String, hash: String) -> Result<bool, AppError>;
}
