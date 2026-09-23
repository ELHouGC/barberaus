use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, infrastructure::jwt::Claims};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    claims: Claims,
    // Future expansion:
    // pub roles: Vec<String>,
    // pub permissions: Vec<String>,
}
#[allow(dead_code)]
impl AuthContext {
    pub fn new(
        claims : Claims,
    ) -> Self {
        Self {
            claims: claims,
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.claims.sub
    }

    pub fn active_profile(&self) -> &str {
        &self.claims.active_profile
    }

    pub fn jti(&self) -> &str {
        &self.claims.jti
    }

    pub fn exp(&self) -> usize {
        self.claims.exp
    }
}

// In-memory extraction from Request Extensions
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(ApiError::MissingToken)?;

        Ok(AuthContext::new(claims))
    }
}


