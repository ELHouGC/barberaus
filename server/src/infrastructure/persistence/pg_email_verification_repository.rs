use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::repositories::email_verification_repository::{
        EmailVerification, EmailVerificationRepo,
    },
    error::ApiError,
};

pub struct PostgresEmailVerificationRepo {
    pool: PgPool,
}

impl PostgresEmailVerificationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailVerificationRepo for PostgresEmailVerificationRepo {
    async fn create(&self, verification: EmailVerification) -> Result<EmailVerification, ApiError> {
        let record = sqlx::query_as::<_, EmailVerificationRecord>(
            r#"
            INSERT INTO email_verifications (
                id, user_id, token_hash, expires_at, verified_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, token_hash, expires_at, verified_at, created_at
            "#,
        )
        .bind(verification.id)
        .bind(verification.user_id)
        .bind(&verification.token_hash)
        .bind(verification.expires_at)
        .bind(verification.verified_at)
        .bind(verification.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            tracing::error!("Failed to insert email verification: {:?}", err);
            ApiError::InternalServerError
        })?;

        Ok(record.into())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<EmailVerification, ApiError> {
        let record = sqlx::query_as::<_, EmailVerificationRecord>(
            r#"
            SELECT id, user_id, token_hash, expires_at, verified_at, created_at
            FROM email_verifications
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        record
            .map(Into::into)
            .ok_or(ApiError::InvalidVerificationToken)
    }

    async fn find_pending_by_user(&self, user_id: Uuid) -> Result<EmailVerification, ApiError> {
        let record = sqlx::query_as::<_, EmailVerificationRecord>(
            r#"
            SELECT id, user_id, token_hash, expires_at, verified_at, created_at
            FROM email_verifications
            WHERE user_id = $1 AND verified_at IS NULL
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        record
            .map(Into::into)
            .ok_or(ApiError::VerificationTokenNotFound)
    }

    async fn find_by_token_hash_and_user_id(
        &self,
        user_id: &Uuid,
        token_hash: &str,
    ) -> Result<EmailVerification, ApiError> {
        let record = sqlx::query_as::<_, EmailVerificationRecord>(
            r#"
            SELECT id, user_id, token_hash, expires_at, verified_at, created_at
            FROM email_verifications
            WHERE user_id = $1 AND token_hash = $2
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?
        .ok_or(ApiError::InvalidVerificationToken)?;

        Ok(record.into())
    }

    async fn verify_and_activate_user(&self, id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| ApiError::InternalServerError)?;

        let verification_result = sqlx::query(
            r#"
            UPDATE email_verifications
            SET verified_at = $2
            WHERE id = $1 AND verified_at IS NULL
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        if verification_result.rows_affected() != 1 {
            return Err(ApiError::InvalidVerificationToken);
        }

        let user_result = sqlx::query(
            r#"
            UPDATE user_credentials
            SET is_account_active = true
            WHERE user_id = $1 AND is_account_active = false
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        if user_result.rows_affected() != 1 {
            return Err(ApiError::AccountAlreadyVerified);
        }

        tx.commit()
            .await
            .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }

    async fn replace_expired_verification(
        &self,
        verification_id: Uuid,
        token_hash: &str,
    ) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            UPDATE email_verifications
            SET
                token_hash = $1,
                expires_at = $2,
                verified_at = NULL
            WHERE id = $3
            "#,
        )
        .bind(token_hash)
        .bind(Utc::now() + Duration::hours(24))
        .bind(verification_id)
        .execute(&self.pool)
        .await
        .map_err(|err| {
            tracing::error!("Failed to replace expired verification: {:?}", err);
            ApiError::InternalServerError
        })?;

        Ok(())
    }

    async fn delete_unverified_user(&self, user_id: Uuid) -> Result<(), ApiError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| ApiError::InternalServerError)?;

        sqlx::query("DELETE FROM email_verifications WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| ApiError::InternalServerError)?;

        sqlx::query(
            "DELETE FROM user_credentials WHERE user_id = $1 AND is_account_active = false",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        tx.commit()
            .await
            .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }
}

// Private SQLx mapping DTO to derive FromRow internally inside infrastructure
#[derive(sqlx::FromRow)]
struct EmailVerificationRecord {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expires_at: chrono::DateTime<Utc>,
    verified_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<EmailVerificationRecord> for EmailVerification {
    fn from(r: EmailVerificationRecord) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            token_hash: r.token_hash,
            expires_at: r.expires_at,
            verified_at: r.verified_at,
            created_at: r.created_at,
        }
    }
}
