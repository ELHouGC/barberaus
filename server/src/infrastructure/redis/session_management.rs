use deadpool_redis::{Pool, redis::cmd};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::ApiError,
    infrastructure::jwt,
    transport::dto::auth::{Session, UserPayload},
};

const REFRESH_TOKEN_TTL_SECONDS: usize = 604_800; // 7 days

#[derive(Serialize, Deserialize)]
struct RefreshSessionMeta {
    pub user_id: Uuid,
    pub active_profile: String,
}

/// 1. Mints Access Token (contains internal jti)
/// 2. Generates a new Refresh Token UUID
/// 3. Writes `refresh:{refresh_token}` -> JSON meta to Redis with a 7-day TTL
pub async fn create_session(
    redis_pool: &Pool,
    user_id: Uuid,
    active_profile: &str,
    jwt_secret: &str,
) -> Result<Session, ApiError> {
    let access_token = jwt::mint_token(user_id, active_profile, jwt_secret)?;
    let refresh_token = Uuid::new_v4().to_string();

    let meta = RefreshSessionMeta {
        user_id,
        active_profile: active_profile.to_string(),
    };
    let meta_json = serde_json::to_string(&meta).map_err(|_| ApiError::Unexpected)?;

    let mut conn = redis_pool
        .get()
        .await
        .map_err(|_| ApiError::DatabaseError)?;
    let redis_key = format!("refresh:{}", refresh_token);

    let _: () = cmd("SETEX")
        .arg(&redis_key)
        .arg(REFRESH_TOKEN_TTL_SECONDS)
        .arg(&meta_json)
        .query_async(&mut conn)
        .await
        .map_err(|_| ApiError::DatabaseError)?;

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

/// Rotates a Refresh Token atomically:
/// 1. Reads user session metadata from `refresh:{old_refresh_token}`
/// 2. Deletes the old refresh token (prevents reuse attacks)
/// 3. Mints a brand-new session (new Access Token + new Refresh Token)
pub async fn rotate_session(
    redis_pool: &Pool,
    old_refresh_token: &str,
    jwt_secret: &str,
) -> Result<Session, ApiError> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|_| ApiError::DatabaseError)?;
    let old_key = format!("refresh:{}", old_refresh_token);

    // 1. Retrieve existing session metadata
    let session_json: Option<String> = cmd("GET")
        .arg(&old_key)
        .query_async(&mut conn)
        .await
        .map_err(|_| ApiError::DatabaseError)?;

    let meta_raw = match session_json {
        Some(json) => json,
        None => return Err(ApiError::InvalidToken), // Expired or revoked refresh token
    };

    let meta: RefreshSessionMeta =
        serde_json::from_str(&meta_raw).map_err(|_| ApiError::InvalidToken)?;

    // 2. Delete old refresh token immediately (Atomicity / Reuse Prevention)
    let _: () = cmd("DEL")
        .arg(&old_key)
        .query_async(&mut conn)
        .await
        .map_err(|_| ApiError::DatabaseError)?;

    // 3. Issue a fresh session (New JWT + New Refresh Token)
    create_session(redis_pool, meta.user_id, &meta.active_profile, jwt_secret).await
}

/// Fully terminates a session on logout:
/// 1. Blacklists current Access Token (`token_blacklist:{jti}`)
/// 2. Deletes Refresh Token (`refresh:{refresh_token}`)
pub async fn terminate_session(
    redis_pool: &Pool,
    _user_id: Uuid,
    jti: &str,
    exp: usize,
    refresh_token: Option<&str>,
) -> Result<(), ApiError> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|_| ApiError::DatabaseError)?;

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
        .map_err(|_| ApiError::DatabaseError)?;

    // B. Delete Refresh Token
    if let Some(rt) = refresh_token {
        let refresh_key = format!("refresh:{}", rt);
        let _: () = cmd("DEL")
            .arg(&refresh_key)
            .query_async(&mut conn)
            .await
            .map_err(|_| ApiError::DatabaseError)?;
    }

    Ok(())
}
