use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct RefreshToken(pub String);
#[allow(dead_code)]
impl RefreshToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<S> FromRequestParts<S> for RefreshToken
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Try extracting from custom `X-Refresh-Token` header
        if let Some(token) = parts
            .headers
            .get("x-refresh-token")
            .and_then(|h| h.to_str().ok())
        {
            if !token.is_empty() {
                return Ok(RefreshToken(token.to_string()));
            }
        }
        println!("Extracting refresh token from request parts: {:?}", parts);
        // 2. Try extracting from Cookie (`refresh_token=<token>`)
        if let Some(cookie_header) = parts
            .headers
            .get(header::COOKIE)
            .and_then(|h| h.to_str().ok())
        {
            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if let Some(token) = cookie.strip_prefix("refresh_token=") {
                    if !token.is_empty() {
                        println!("Extracted refresh token from cookie: {}", token);
                        return Ok(RefreshToken(token.to_string()));
                    }
                }
                println!("Extracting refresh token was empty");
            }
        }

        Err(ApiError::MissingToken)
    }
}
