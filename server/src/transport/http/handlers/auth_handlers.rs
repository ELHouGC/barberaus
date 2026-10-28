// auth_handlers.rs
//
// STYLE RULES FOR THIS FILE
// -------------------------
// 1. Handlers are THIN: extract input -> call one service -> shape the response.
//    No business rules here, no error construction: `?` converts everything to `AppError`.
// 2. Every handler returns `Result<_, AppError>`. `problem.rs` turns the error into
//    the RFC 9457 response and logs it once. (Mixing `ApiError` and `AppError` in the
//    same file would not compile: there is only one error type at the HTTP boundary.)
// 3. Logging: `tracing` only, never `println!`, and NEVER a token, cookie or email.
//    We log business events with the user id (`info!`), nothing else.
// 4. What `login`, `verify_email` and `refresh_token` have in common (cookie + JSON
//    body) lives in two small private helpers at the top, written exactly once.

use axum::{Json, extract::State, http::HeaderMap};
use uuid::Uuid;

use crate::{
    application::error::AppError,
    infrastructure::redis::session_management,
    state::AppState,
    transport::{
        dto::auth::{
            AuthResponse, LoginRequest, MessageResponse, ResendVerificationRequest, Session,
            SignupRequest, SignupResponse, VerifyEmailRequest,
        },
        middlewares::{auth_context::AuthContext, auth_cookies, refresh_token::RefreshToken},
        response::ApiResponse,
    },
};

/// What every "you now have a session" endpoint returns:
/// the `Set-Cookie` header (refresh token) + the JSON body (access token + user).
/// A type alias is just a shorter name; it does not create a new type.
type SessionReply = (HeaderMap, Json<ApiResponse<AuthResponse>>);

/// Turns a freshly created/rotated `Session` into the HTTP reply.
///
/// The refresh token goes ONLY in the HttpOnly cookie; the body never contains it.
fn session_reply(session: Session) -> Result<SessionReply, AppError> {
    // `&session.refresh_token` only borrows, so we can still move the other
    // fields out of `session` below.
    let headers = auth_cookies::create_refresh_cookie(&session.refresh_token)?;

    Ok((
        headers,
        Json(ApiResponse::success(AuthResponse {
            access_token: session.access_token,
            token_type: session.token_type,
            user: session.user,
        })),
    ))
}

/// Creates the dual-token session in Redis, then builds the reply.
/// Shared by `login` and `verify_email` (it used to be copy-pasted in both).
async fn open_session(
    state: &AppState,
    user_id: Uuid,
    is_barber_profile_active: bool,
) -> Result<SessionReply, AppError> {
    // Plain `if/else` instead of `.then(|| "BARBER").unwrap_or("CLIENT")`:
    // same result, but it reads like the sentence it expresses.
    let active_profile = if is_barber_profile_active {
        "BARBER"
    } else {
        "CLIENT"
    };

    let session = session_management::create_session(
        &state.redis_pool,
        user_id,
        active_profile,
        &state.config.jwt_secret,
    )
    .await?; // Redis problems arrive here already converted to AppError

    session_reply(session)
}

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<ApiResponse<SignupResponse>>, AppError> {
    payload.validate()?;
    let res = state.auth_app.signup(payload).await?;
    Ok(Json(ApiResponse::success(res)))
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<SessionReply, AppError> {
    // 1. Verify the email code in the DB (the service logs the business event).
    let res = state.auth_app.verify_email(payload).await?;

    // 2. Open the session: Redis tokens + Set-Cookie + JSON body.
    open_session(&state, res.user_id, res.is_barber_profile_active).await
}

pub async fn resend_verification(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, AppError> {
    let res = state.auth_app.resend_verification(payload).await?;
    Ok(Json(ApiResponse::success(res)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<SessionReply, AppError> {
    // 1. Check credentials. A wrong password / unverified account is an `Err` here.
    let res = state.auth_app.login(payload).await?;

    // 2. Open the session.
    let reply = open_session(&state, res.user_id, res.is_barber_profile_active).await?;

    // 3. Log AFTER success only, so the log never claims a login that did not complete.
    //    User id only: never the email, the password or any token.
    tracing::info!(user_id = %res.user_id, "user logged in");

    Ok(reply)
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthContext,
    refresh_token: RefreshToken,
) -> Result<(HeaderMap, Json<&'static str>), AppError> {
    // 1. Blacklist the access token and delete the refresh token in Redis.
    session_management::terminate_session(
        &state.redis_pool,
        auth.user_id(),
        auth.jti(),
        auth.exp(),
        Some(refresh_token.as_str()),
    )
    .await?;

    tracing::info!(user_id = %auth.user_id(), "user logged out");

    // 2. Tell the browser to delete the cookie, and return (headers, body):
    //    Axum turns the tuple into the HTTP response.
    Ok((
        auth_cookies::clear_refresh_cookie(),
        Json("Successfully logged out"),
    ))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    refresh_token: RefreshToken,
) -> Result<SessionReply, AppError> {
    // Rotate: the old refresh token is invalidated, a new pair is issued.
    // (The old `println!`s printed BOTH the old and the new refresh token.)
    let session =
        session_management::rotate_session(&state.redis_pool, refresh_token.as_str(), &state.config.jwt_secret)
            .await?;
    tracing::info!("session rotated");

    session_reply(session)
}