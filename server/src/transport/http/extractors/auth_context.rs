// transport/http/auth_context.rs
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{application::error::AppError, infrastructure::jwt::Claims};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub claims: Claims,
    // Future expansion:
    // pub roles: Vec<String>,
    // pub permissions: Vec<String>,
}
#[allow(dead_code)]
impl AuthContext {
    pub fn new(claims: Claims) -> Self {
        // Field init shorthand: `claims` instead of `claims: claims`.
        Self { claims }
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

// In-memory extraction from request extensions (filled by `require_auth`).
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // WHY THIS IS AN INTERNAL ERROR AND NOT `MissingToken` (401):
        // `require_auth` already rejected every request without a valid token.
        // If the claims are absent HERE, the client did nothing wrong: a route
        // using `AuthContext` was registered WITHOUT the `require_auth` layer.
        // That is OUR bug, so it must be a 500 that is logged at error level,
        // not a 401 that sends developers debugging the client.
        // (Access is still denied either way: fail-closed.)
        let claims = parts.extensions.get::<Claims>().cloned().ok_or_else(|| {
            AppError::internal("AuthContext used on a route without the require_auth middleware")
        })?;

        Ok(AuthContext::new(claims))
    }
}