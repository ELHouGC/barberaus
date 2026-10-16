// application/services/email_verification.rs
//
// STYLE RULES FOR THIS FILE
// -------------------------
// 1. The service is where "nothing found" gets a business meaning:
//    - for signup it means "create a code"  (not an error at all),
//    - for verify it means "invalid token"  (-> AppError::Verification).
// 2. The service returns `AppError`. Repository failures are converted
//    automatically by `?` thanks to `impl From<RepoError> for AppError`.
// 3. The service logs important BUSINESS events (`info!`), never errors:
//    errors are logged once in `problem.rs`.
// 4. Never log emails, codes or hashes (personal data / secrets).

use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::{
    error::{AppError, AuthError, ConflictKind, VerificationError},
    ports::email_verification_repository::{
        ActivationOutcome, EmailVerification, EmailVerificationRepo,
    },
};

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
}

/// Adapters implementing this trait should report a provider outage as
/// `AppError::Unavailable { dependency: "email_provider", source }` (-> HTTP 503)
/// and anything unexpected as `AppError::internal(e)` (-> HTTP 500).
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, message: EmailMessage) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct EmailVerificationService {
    repo: Arc<dyn EmailVerificationRepo>,
    pub email_sender: Arc<dyn EmailSender>,
    pub email_registration_from: String,
}

impl EmailVerificationService {
    pub fn new(
        repo: Arc<dyn EmailVerificationRepo>,
        email_sender: Arc<dyn EmailSender>,
        from_email: String,
    ) -> Self {
        Self {
            repo,
            email_sender,
            email_registration_from: from_email,
        }
    }

    fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let code: u32 = rng.gen_range(100_000..999_999);
        code.to_string()
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Primary entry point for Signup flow.
    // `skip_all` + explicit field: we record `user_id` but NOT `user_email` (personal data).
    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn prepare_for_verification(
        &self,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<(), AppError> {
        // `.await?` -> if the repo failed, a RepoError becomes an AppError and we return.
        // Otherwise we get an `Option<EmailVerification>` to match on.
        match self.repo.find_pending_by_user(user_id).await? {
            // `if ...` after a pattern is a "match guard": this arm only matches
            // when the condition is true. Code is still valid -> nothing to do.
            Some(verification) if verification.expires_at > Utc::now() => Ok(()),

            // A pending code exists but it is expired -> replace it and resend.
            Some(verification) => {
                self.regenerate_and_send_code(verification.id, user_email)
                    .await
            }

            // No pending code. Before, this was an `Err(VerificationTokenNotFound)`
            // that we had to catch; now it is just a normal case.
            None => {
                let raw_token = Self::generate_token();
                let token_hash = Self::hash_token(&raw_token);
                let expires_at = Utc::now() + Duration::hours(24);

                let verification = EmailVerification::new(user_id, token_hash, expires_at);

                self.repo.create(verification).await?;
                self.send_code(user_email, &raw_token).await
            }
        }
    }

    /// Dedicated entry point for Login guard when `!is_account_active`.
    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn handle_unverified_login_attempt(
        &self,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<(), AppError> {
        match self.repo.find_pending_by_user(user_id).await? {
            // Token is active & valid -> Ok, the handler answers 403 AccountNotVerified.
            Some(verification) if verification.expires_at > Utc::now() => Ok(()),

            // Token is expired -> regenerate, update DB, and resend email.
            Some(verification) => {
                self.regenerate_and_send_code(verification.id, user_email)
                    .await
            }

            // Token record was purged by the 72h worker or is missing -> delete the orphaned user.
            None => {
                self.repo.delete_unverified_user(user_id).await?;

                // A business event worth a log line (note: no email in it).
                tracing::info!("orphaned unverified user deleted: no pending verification");

                // Fail with InvalidCredentials so the UI prompts re-registration.
                // Here `Err` is intentional control flow, not a technical failure.
                Err(AppError::Auth(AuthError::InvalidCredentials))
            }
        }
    }

    async fn send_code(&self, email: &str, raw_token: &str) -> Result<(), AppError> {
        let message = EmailMessage {
            from: self.email_registration_from.clone(),
            to: vec![email.to_string()],
            subject: "BarberOS Email Verification".to_string(),
            text_body: Some(format!("Your verification code is: {}", raw_token)),
            html_body: None,
        };
        self.email_sender.send(message).await
    }

    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn verify(&self, user_id: Uuid, raw_token: &str) -> Result<(), AppError> {
        let token_hash = Self::hash_token(raw_token);

        // `None` here means "no such code for this user" -> business meaning: invalid token.
        // `.ok_or(x)` turns `Option<T>` into `Result<T, X>`: `Some(v)` -> `Ok(v)`, `None` -> `Err(x)`.
        let verification = self
            .repo
            .find_by_token_hash_and_user_id(&user_id, &token_hash)
            .await?
            .ok_or(AppError::Verification(VerificationError::InvalidToken))?;

        if verification.verified_at.is_some() {
            return Err(AppError::Verification(VerificationError::InvalidToken));
        }

        if verification.expires_at < Utc::now() {
            return Err(AppError::Verification(VerificationError::Expired));
        }

        // The repo reports WHAT happened; the service decides which error it is.
        // `match` forces us to handle every variant: add a new outcome and the
        // compiler tells us where we forgot it.
        match self
            .repo
            .verify_and_activate_user(verification.id, user_id)
            .await?
        {
            ActivationOutcome::Activated => {
                tracing::info!("email verified, account activated");
                Ok(())
            }
            ActivationOutcome::VerificationAlreadyUsed => {
                Err(AppError::Verification(VerificationError::InvalidToken))
            }
            ActivationOutcome::AccountAlreadyActive => {
                Err(AppError::Conflict(ConflictKind::AccountAlreadyVerified))
            }
        }
    }

    async fn regenerate_and_send_code(
        &self,
        verification_id: Uuid,
        user_email: &str,
    ) -> Result<(), AppError> {
        let raw_token = Self::generate_token();
        let token_hash = Self::hash_token(&raw_token);

        self.repo
            .replace_expired_verification(verification_id, &token_hash)
            .await?;

        self.send_code(user_email, &raw_token).await
    }
}