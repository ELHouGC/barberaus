// infrastructure/redis/session_management.rs
//
// STYLE RULES FOR THIS FILE
// -------------------------
// 1. Pure refactor: same Redis commands, same behaviour. Only the error handling changes.
//    (Two behaviour improvements are listed at the bottom as separate commits.)
// 2. WHO IS AT FAULT decides the error:
//      Redis unreachable / command failed -> AppError::Unavailable (503, our dependency)
//      refresh token unknown or revoked   -> AuthError::InvalidToken (401, client's token)
//      our own data is corrupt / a bug    -> AppError::internal (500)
// 3. No logging of errors here: they travel up and are logged once in `problem.rs`.
//    NEVER put a refresh token (or its Redis key) in a log or a span field.

use deadpool_redis::{Pool, redis::cmd};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::error::{AppError, AuthError},
    infrastructure::jwt,
    transport::dto::auth::{Session, UserPayload},
};

const REFRESH_TOKEN_TTL_SECONDS: usize = 604_800; // 7 days

/// Any Redis failure (pool exhausted, connection lost, command error) means the
/// same thing for the caller: "our session store is not working".
///
/// This replaces the `|e| AppError::unavailable("redis", e)` closure that would
/// otherwise be repeated on every Redis call. Use it as `.map_err(redis_down)`.
fn redis_down<E>(e: E) -> AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    AppError::unavailable("redis", e)
}

#[derive(Serialize, Deserialize)]
struct RefreshSessionMeta {
    pub user_id: Uuid,
    pub active_profile: String,
}

/// 1. Mints Access Token (contains internal jti)
/// 2. Generates a new Refresh Token UUID
/// 3. Writes `refresh:{refresh_token}` -> JSON meta to Redis with a 7-day TTL
// `skip_all` + explicit field: the span knows the user, but can never capture a secret.
#[tracing::instrument(level = "debug", skip_all, fields(user_id = %user_id))]
pub async fn create_session(
    redis_pool: &Pool,
    user_id: Uuid,
    active_profile: &str,
    jwt_secret: &str,
) -> Result<Session, AppError> {
    // `mint_token` must return `AppError` too (signing failure = our bug = internal).
    let access_token = jwt::mint_token(user_id, active_profile, jwt_secret)?;
    let refresh_token = Uuid::new_v4().to_string();

    let meta = RefreshSessionMeta {
        user_id,
        active_profile: active_profile.to_string(),
    };
    // Serializing a plain struct cannot fail unless we have a bug -> internal error.
    // `AppError::internal` keeps the original serde error for the logs.
    let meta_json = serde_json::to_string(&meta).map_err(AppError::internal)?;

    let mut conn = redis_pool.get().await.map_err(redis_down)?;
    let redis_key = format!("refresh:{}", refresh_token);

    let _: () = cmd("SETEX")
        .arg(&redis_key)
        .arg(REFRESH_TOKEN_TTL_SECONDS)
        .arg(&meta_json)
        .query_async(&mut conn)
        .await
        .map_err(redis_down)?;

    let is_barber_profile_active = active_profile.eq_ignore_ascii_case("BARBER");

    Ok(Session {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user: UserPayload {
            user_id,
            is_barber_profile_active,
        },
    })
}

/// Rotates a Refresh Token:
/// 1. Reads user session metadata from `refresh:{old_refresh_token}`
/// 2. Deletes the old refresh token (prevents reuse attacks)
/// 3. Mints a brand-new session (new Access Token + new Refresh Token)
// No fields at all: the only identifier we have here is the secret itself.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn rotate_session(
    redis_pool: &Pool,
    old_refresh_token: &str,
    jwt_secret: &str,
) -> Result<Session, AppError> {
    let mut conn = redis_pool.get().await.map_err(redis_down)?;
    let old_key = format!("refresh:{}", old_refresh_token);

    // 1. Retrieve existing session metadata
    let session_json: Option<String> = cmd("GET")
        .arg(&old_key)
        .query_async(&mut conn)
        .await
        .map_err(redis_down)?;

    // `None` = expired or revoked refresh token. That is the CLIENT's problem -> 401.
    // `ok_or` turns `Option<T>` into `Result<T, E>`: `Some(v)` -> `Ok(v)`, `None` -> `Err(E)`.
    let meta_raw = session_json.ok_or(AppError::Auth(AuthError::InvalidToken))?;

    // Unparsable JSON means OUR data in Redis is corrupt (we wrote it). Telling the
    // client "invalid token" would hide a real bug, so this is an internal error (500).
    // (Before: InvalidToken. This is a deliberate reclassification.)
    let meta: RefreshSessionMeta = serde_json::from_str(&meta_raw).map_err(AppError::internal)?;

    // 2. Delete old refresh token immediately (reuse prevention)
    let _: () = cmd("DEL")
        .arg(&old_key)
        .query_async(&mut conn)
        .await
        .map_err(redis_down)?;

    // 3. Issue a fresh session (new JWT + new refresh token)
    create_session(redis_pool, meta.user_id, &meta.active_profile, jwt_secret).await
}

/// Fully terminates a session on logout:
/// 1. Blacklists current Access Token (`token_blacklist:{jti}`)
/// 2. Deletes Refresh Token (`refresh:{refresh_token}`)
#[tracing::instrument(level = "debug", skip_all, fields(user_id = %_user_id))]
pub async fn terminate_session(
    redis_pool: &Pool,
    _user_id: Uuid,
    jti: &str,
    exp: usize,
    refresh_token: Option<&str>,
) -> Result<(), AppError> {
    let mut conn = redis_pool.get().await.map_err(redis_down)?;

    // A. Blacklist active Access Token
    let now = chrono::Utc::now().timestamp() as usize;
    let ttl_seconds = if exp > now { exp - now } else { 1 };
    let blacklist_key = format!("token_blacklist:{}", jti);

    let _: () = cmd("SETEX")
        .arg(&blacklist_key)
        .arg(ttl_seconds)
        .arg("1")
        .query_async(&mut conn)
        .await
        .map_err(redis_down)?;

    // B. Delete Refresh Token
    if let Some(rt) = refresh_token {
        let refresh_key = format!("refresh:{}", rt);
        let _: () = cmd("DEL")
            .arg(&refresh_key)
            .query_async(&mut conn)
            .await
            .map_err(redis_down)?;
    }

    Ok(())
}

// FOLLOW-UP (separate commit, behaviour change):
// `rotate_session` does GET then DEL: two round-trips, NOT atomic, despite what the doc says.
// Two concurrent refreshes with the same token can both succeed and issue two sessions.
// Redis >= 6.2 has `GETDEL` (read + delete in one atomic command): use it and drop the DEL.
