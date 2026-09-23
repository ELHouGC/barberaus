use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::repositories::email_verification_repository::{
        EmailVerification, EmailVerificationRepo,
    },
    error::ApiError,
};

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
}

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, message: EmailMessage) -> Result<(), ApiError>;
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

    /// Primary entry point for Signup flow
    pub async fn prepare_for_verification(
        &self,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<(), ApiError> {
        match self.repo.find_pending_by_user(user_id).await {
            Ok(verification) => {
                if verification.expires_at > Utc::now() {
                    return Ok(());
                }

                self.regenerate_and_send_code(verification.id, user_email)
                    .await
            }

            Err(ApiError::VerificationTokenNotFound) => {
                let raw_token = Self::generate_token();
                let token_hash = Self::hash_token(&raw_token);
                let expires_at = Utc::now() + Duration::hours(24);

                let verification = EmailVerification::new(user_id, token_hash, expires_at);

                self.repo.create(verification).await?;
                self.send_code(user_email, &raw_token).await
            }

            Err(err) => Err(err),
        }
    }

    /// Dedicated entry point for Login guard when `!is_account_active`
    pub async fn handle_unverified_login_attempt(
        &self,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<(), ApiError> {
        match self.repo.find_pending_by_user(user_id).await {
            Ok(verification) => {
                if verification.expires_at > Utc::now() {
                    // Token is active & valid -> proceed to return 403 in handler
                    return Ok(());
                }

                // Token is expired -> regenerate, update DB, and resend email
                self.regenerate_and_send_code(verification.id, user_email)
                    .await
            }

            Err(ApiError::VerificationTokenNotFound) => {
                // Token record was purged by 72h worker or missing -> delete orphaned user
                self.repo.delete_unverified_user(user_id).await?;
                // Fail with InvalidCredentials so UI prompts re-registration
                Err(ApiError::InvalidCredentials)
            }

            Err(err) => Err(err),
        }
    }

    async fn send_code(&self, email: &str, raw_token: &str) -> Result<(), ApiError> {
        let message = EmailMessage {
            from: self.email_registration_from.clone(),
            to: vec![email.to_string()],
            subject: "BarberOS Email Verification".to_string(),
            text_body: Some(format!("Your verification code is: {}", raw_token)),
            html_body: None,
        };
        self.email_sender.send(message).await
    }

    pub async fn verify(&self, user_id: Uuid, raw_token: &str) -> Result<(), ApiError> {
        let token_hash = Self::hash_token(raw_token);

        let verification = self
            .repo
            .find_by_token_hash_and_user_id(&user_id, &token_hash)
            .await?;

        if verification.verified_at.is_some() {
            return Err(ApiError::InvalidVerificationToken);
        }

        if verification.expires_at < Utc::now() {
            return Err(ApiError::VerificationTokenExpired);
        }

        self.repo
            .verify_and_activate_user(verification.id, user_id)
            .await?;

        Ok(())
    }

    async fn regenerate_and_send_code(
        &self,
        verification_id: Uuid,
        user_email: &str,
    ) -> Result<(), ApiError> {
        let raw_token = Self::generate_token();
        let token_hash = Self::hash_token(&raw_token);

        self.repo
            .replace_expired_verification(verification_id, &token_hash)
            .await?;

        self.send_code(user_email, &raw_token).await
    }
}
