// application/ports/email_verification_repository.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::repo_error::RepoError;

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

/// Result of `verify_and_activate_user`.
///
/// "The verification was already used" is a *normal business outcome*, not a
/// technical failure, so it cannot be a `RepoError`. And the repository must not
/// return an `AppError` (that would leak HTTP concerns into the data layer).
/// So the repository *reports what happened* and the service decides the error.
#[derive(Debug, PartialEq, Eq)]
pub enum ActivationOutcome {
    /// Verification marked as used AND account activated (transaction committed).
    Activated,
    /// The verification row was not pending (already used). Nothing was changed.
    VerificationAlreadyUsed,
    /// No inactive account matched (already active, or gone). Nothing was changed.
    AccountAlreadyActive,
}

// Conventions used by every method of this port:
//   * `Result<_, RepoError>`          -> technical failure only.
//   * `Result<Option<T>, RepoError>`  -> "a lookup that may find nothing".
//     `None` is NOT an error here: whether "nothing found" is a problem
//     depends on the caller, so the service decides.
#[allow(dead_code)]
#[async_trait]
pub trait EmailVerificationRepo: Send + Sync {
    async fn create(&self, verification: EmailVerification)
        -> Result<EmailVerification, RepoError>;

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<EmailVerification>, RepoError>;

    async fn find_pending_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Option<EmailVerification>, RepoError>;

    async fn find_by_token_hash_and_user_id(
        &self,
        user_id: &Uuid,
        token_hash: &str,
    ) -> Result<Option<EmailVerification>, RepoError>;

    async fn verify_and_activate_user(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<ActivationOutcome, RepoError>;

    async fn replace_expired_verification(
        &self,
        verification_id: Uuid,
        token_hash: &str,
    ) -> Result<(), RepoError>;

    async fn delete_unverified_user(&self, user_id: Uuid) -> Result<(), RepoError>;
}