// transport/http/middleware/auth.rs
//
// STYLE RULES FOR THIS FILE
// -------------------------
// 1. Every failure is an `AppError`. `problem.rs` turns it into the HTTP response.
// 2. Distinguish WHO is at fault:
//      - client sent a bad/missing/revoked token  -> AppError::Auth(..)     (401)
//      - Redis is down / unreachable              -> AppError::Unavailable  (503)
//    Telling the client "invalid token" when OUR Redis is down would be a lie,
//    and it would hide the outage from our monitoring.
// 3. Never log the token itself. Log who (user id) and what happened.

use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use deadpool_redis::redis::cmd;

use crate::{
    application::error::{AppError, AuthError},
    infrastructure::jwt,
    state::AppState,
};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. Extract the Authorization header.
    //    Three different situations, three different outcomes:
    //      header absent                  -> MissingToken
    //      header present but not text    -> InvalidToken
    //      header present, not "Bearer x" -> InvalidToken
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or(AppError::Auth(AuthError::MissingToken))?;

    let header = header
        .to_str()
        .map_err(|_| AppError::Auth(AuthError::InvalidToken))?;

    // `strip_prefix` returns `Some(rest)` only if the prefix is there.
    // It replaces `starts_with` + `&header[7..]` (no magic number 7).
    let token = header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Auth(AuthError::InvalidToken))?;

    // 2. Decode JWT signature & expiration.
    //    `decode_token` must return `AppError` (see the jwt.rs mapping in the guide).
    let claims = jwt::decode_token(token, &state.config.jwt_secret)?;

    // 3. Check the Redis blacklist (fail-closed: if we cannot check, we refuse).
    //    `AppError::unavailable("redis", e)` keeps the original error for the logs
    //    and answers 503 to the client.
    let mut redis_conn = state
        .redis_pool
        .get()
        .await
        .map_err(|e| AppError::unavailable("redis", e))?;

    let is_blacklisted: bool = cmd("EXISTS")
        .arg(format!("token_blacklist:{}", claims.jti))
        .query_async(&mut redis_conn)
        .await
        .map_err(|e| AppError::unavailable("redis", e))?;

    if is_blacklisted {
        // A revoked token being replayed is worth a trace (note: user id only, never the token).
        tracing::warn!(user_id = %claims.sub, "rejected request using a blacklisted token");
        return Err(AppError::Auth(AuthError::InvalidToken));
    }

    // 4. Inject the claims into the request extensions for downstream extractors
    //    (`AuthContext` reads them). We MOVE `claims` here: the old `.clone()`
    //    was useless because `claims` is not used afterwards.
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}