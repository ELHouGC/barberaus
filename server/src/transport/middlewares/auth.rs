use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deadpool_redis::redis::cmd;

use crate::{
    error::ApiError,
    infrastructure::jwt,
    state::AppState,

};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // 1. Extract Authorization Header
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::MissingToken)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::InvalidToken);
    }

    let token = &auth_header[7..];

    // 2. Decode JWT Signature & Expiration
    let claims = jwt::decode_token(token, &state.config.jwt_secret)?;

    // 3. Check Redis Blacklist (Fail-closed)
    let mut redis_conn = state
        .redis_pool
        .get()
        .await
        .map_err(|_| ApiError::DatabaseError)?;

    let is_blacklisted: bool = cmd("EXISTS")
        .arg(format!("token_blacklist:{}", claims.jti))
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| ApiError::DatabaseError)?;

    if is_blacklisted {
        return Err(ApiError::InvalidToken);
    }


  

    // 5. Inject AuthContext into Request Extensions for downstream handlers
    req.extensions_mut().insert(claims.clone());

    Ok(next.run(req).await)
}