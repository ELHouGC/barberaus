use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::application::{ error::{AppError, ConflictKind, Resource}, ports::auth_repository::{UserAuthRepository, UserCredentialsRecord }};

pub struct PgUserAuthRepository {
    pool: PgPool,
}

impl PgUserAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
        
#[async_trait::async_trait]
impl UserAuthRepository for PgUserAuthRepository {
    async fn create_user(&self, record: UserCredentialsRecord) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO user_credentials (user_id, email, password_hash, full_name, is_barber_profile_active, is_account_active)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(record.user_id)
        .bind(record.email)
        .bind(record.password_hash)
        .bind(record.full_name)
        .bind(record.is_barber_profile_active)
        .bind(record.is_account_active)
        .execute(&self.pool)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result {
            if db_err.is_unique_violation() {
                return Err(AppError::Conflict(ConflictKind::EmailExists));
            }
        }

        result.map_err(|e| {
            tracing::error!(error = ?e, "Database error while creating user credentials");
            AppError::Unavailable { 
                dependency: "database", 
                source: Box::new(e) 
            }
        })?;

        Ok(())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserCredentialsRecord>, AppError> {
        sqlx::query_as::<_, UserCredentialsRecord>(
            r#"
            SELECT user_id, email, password_hash, full_name, is_account_active, is_barber_profile_active
            FROM user_credentials
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "Database error while finding user by email");
            AppError::Unavailable { 
                dependency: "database", 
                source: Box::new(e) 
            }
        })
    }
    async fn delete_user(&self, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM user_credentials WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, user_id = %user_id, "Database error while deleting user");
                AppError::Unavailable { 
                    dependency: "database", 
                    source: Box::new(e) 
                }
            })?;

        // Check if any row was actually deleted
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(Resource::User));
        }

        Ok(())
    }

    async fn resolve_tenant_role(
        &self,
        user_id: Uuid,
        tenant_id: &str,
    ) -> Result<Option<String>, AppError> {
        let role = sqlx::query(
            r#"
            SELECT role
            FROM user_shop_memberships
            WHERE user_id = $1 AND tenant_id = $2 AND is_active = TRUE
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, user_id = %user_id, tenant_id = %tenant_id, "Database error while resolving tenant role");
            AppError::Unavailable { 
                dependency: "database", 
                source: Box::new(e) 
            }
        })?;

        role.map(|row| {
            row.try_get("role").map_err(|e| {
                tracing::error!(error = ?e, "Failed to parse role column from membership row");
                AppError::internal(e)
            })
        })
        .transpose()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserCredentialsRecord>, AppError> {
        sqlx::query_as::<_, UserCredentialsRecord>(
            r#"
            SELECT user_id, email, password_hash, full_name, is_account_active, is_barber_profile_active
            FROM user_credentials
            WHERE user_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, user_id = %id, "Database error while finding user by id");
            AppError::Unavailable { 
                dependency: "database", 
                source: Box::new(e) 
            }
        })
    }
}