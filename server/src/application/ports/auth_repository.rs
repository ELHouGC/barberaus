
use async_trait::async_trait;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::error::ApiError;


#[derive(Debug, Clone, FromRow)]


pub struct UserCredentialsRecord {
    pub user_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub is_account_active: bool,
    pub is_barber_profile_active: bool,
}


#[allow(dead_code)]
#[async_trait]
pub trait UserAuthRepository: Send + Sync {
    async fn create_user(&self, record: UserCredentialsRecord) -> Result<(), ApiError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<UserCredentialsRecord>, ApiError>;
    async fn delete_user(&self, user_id: Uuid) -> Result<(), ApiError>;

    async fn resolve_tenant_role(
        &self,
        user_id: Uuid,
        tenant_id: &str,
    ) -> Result<Option<String>, ApiError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserCredentialsRecord>, ApiError>;
}
