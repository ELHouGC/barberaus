use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct EmailVerification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl EmailVerification {
    pub fn new(user_id: Uuid, token_hash: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            expires_at,
            verified_at: None,
            created_at: Utc::now(),
        }
    }
}
#[allow(dead_code)]
#[async_trait]

pub trait EmailVerificationRepo: Send + Sync {
    async fn create(&self, verification: EmailVerification) -> Result<EmailVerification, ApiError>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<EmailVerification, ApiError>;
    async fn find_pending_by_user(&self, user_id: Uuid) -> Result<EmailVerification, ApiError>;
    async fn find_by_token_hash_and_user_id(
        &self,
        user_id: &Uuid,
        token_hash: &str,
    ) -> Result<EmailVerification, ApiError>;
    async fn verify_and_activate_user(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError>;
    async fn replace_expired_verification(
        &self,
        verification_id: Uuid,
        token_hash: &str,
    ) -> Result<(), ApiError>;
    async fn delete_unverified_user(&self, user_id: Uuid) -> Result<(), ApiError>;
}
