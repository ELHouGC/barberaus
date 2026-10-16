// infrastructure/postgres/pg_email_verification_repository.rs
//
// STYLE RULES FOR THIS FILE (they apply to every method)
// ------------------------------------------------------
// 1. A repository only talks to the database. It returns data (`Option<T>`)
//    or a technical `RepoError`. It never decides business errors.
// 2. A repository NEVER logs errors. The error travels up and is logged ONCE,
//    in `problem.rs` (`IntoResponse for AppError`). Logging at every layer
//    gives you the same failure 3 times in the logs.
// 3. Every `sqlx` call ends with `.map_err(db_err("<operation>"))?`.
//    This keeps the original sqlx error AND records which operation failed.
//    (The old `|_| ApiError::InternalServerError` threw the real cause away.)
// 4. Tracing: `#[instrument(skip_all, fields(...))]` creates a span per call.
//    We use `skip_all` + explicit fields so secrets (token hashes) and
//    personal data can never be logged by accident.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{application::ports::{
    email_verification_repository::{ActivationOutcome, EmailVerification, EmailVerificationRepo},
    repo_error::RepoError,
}, infrastructure::persistence::db_err};


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
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(verification_id = %verification.id, user_id = %verification.user_id)
    )]
    async fn create(
        &self,
        verification: EmailVerification,
    ) -> Result<EmailVerification, RepoError> {
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
        // `?` = "if this is an Err, return it from the function immediately".
        .map_err(db_err("email_verifications.create"))?;

        Ok(record.into())
    }

    #[tracing::instrument(level = "debug", skip_all)] // no fields: the hash is a secret
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<EmailVerification>, RepoError> {
        let record = sqlx::query_as::<_, EmailVerificationRecord>(
            r#"
            SELECT id, user_id, token_hash, expires_at, verified_at, created_at
            FROM email_verifications
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool) // -> Result<Option<Record>, sqlx::Error>
        .await
        .map_err(db_err("email_verifications.find_by_token_hash"))?;

        // "Not found" is NOT an error at this layer: we return `None` and let
        // the service decide. `Option::map` converts Record -> EmailVerification
        // only when there is a value.
        Ok(record.map(Into::into))
    }

    #[tracing::instrument(level = "debug", skip_all, fields(user_id = %user_id))]
    async fn find_pending_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Option<EmailVerification>, RepoError> {
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
        .map_err(db_err("email_verifications.find_pending_by_user"))?;

        Ok(record.map(Into::into))
    }

    #[tracing::instrument(level = "debug", skip_all, fields(user_id = %user_id))]
    async fn find_by_token_hash_and_user_id(
        &self,
        user_id: &Uuid,
        token_hash: &str,
    ) -> Result<Option<EmailVerification>, RepoError> {
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
        .map_err(db_err("email_verifications.find_by_token_hash_and_user_id"))?;

        Ok(record.map(Into::into))
    }

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(verification_id = %id, user_id = %user_id)
    )]
    async fn verify_and_activate_user(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<ActivationOutcome, RepoError> {
        // Both updates must succeed together or not at all -> one transaction.
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(db_err("verify_and_activate_user.begin"))?;

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
        .map_err(db_err("verify_and_activate_user.mark_verified"))?;

        // IMPORTANT for beginners: returning early here drops `tx` without calling
        // `commit()`. sqlx then ROLLS BACK automatically. So every early `return`
        // below leaves the database untouched.
        //
        // This is a business outcome, not a failure -> `Ok(<outcome>)`, not `Err`.
        if verification_result.rows_affected() != 1 {
            return Ok(ActivationOutcome::VerificationAlreadyUsed);
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
        .map_err(db_err("verify_and_activate_user.activate_user"))?;

        if user_result.rows_affected() != 1 {
            // Rolls back the first UPDATE too (same behaviour as before).
            return Ok(ActivationOutcome::AccountAlreadyActive);
        }

        tx.commit()
            .await
            .map_err(db_err("verify_and_activate_user.commit"))?;

        Ok(ActivationOutcome::Activated)
    }

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(verification_id = %verification_id)
    )]
    async fn replace_expired_verification(
        &self,
        verification_id: Uuid,
        token_hash: &str,
    ) -> Result<(), RepoError> {
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
        // The old inline `tracing::error!` is gone: the error is logged once, at the HTTP boundary.
        .map_err(db_err("email_verifications.replace_expired"))?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(user_id = %user_id))]
    async fn delete_unverified_user(&self, user_id: Uuid) -> Result<(), RepoError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(db_err("delete_unverified_user.begin"))?;

        sqlx::query("DELETE FROM email_verifications WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(db_err("delete_unverified_user.delete_verifications"))?;

        sqlx::query(
            "DELETE FROM user_credentials WHERE user_id = $1 AND is_account_active = false",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(db_err("delete_unverified_user.delete_credentials"))?;

        tx.commit()
            .await
            .map_err(db_err("delete_unverified_user.commit"))?;

        Ok(())
    }
}

// Private SQLx mapping DTO: only this file knows the table shape.
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