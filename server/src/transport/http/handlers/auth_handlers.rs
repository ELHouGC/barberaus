use axum::{Json, extract::State, http::HeaderMap};

use crate::{
    error::ApiError,
    infrastructure::redis::session_management::{self},
    state::AppState,
    transport::{
        dto::auth::{
            AuthResponse, CompleteOnboardingRequest, LoginRequest, MessageResponse,
            OnboardingResponse, ResendVerificationRequest, Session, SignupRequest, SignupResponse,
            VerifyEmailRequest,
        },
        middlewares::{auth_context::AuthContext, auth_cookies, refresh_token::RefreshToken},
        response::ApiResponse,
    },
};

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<ApiResponse<SignupResponse>>, ApiError> {
    payload.validate()?;
    let res = state.auth_app.signup(payload).await?;
    Ok(Json(ApiResponse::success(res)))
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<(HeaderMap, Json<ApiResponse<AuthResponse>>), ApiError> {
    // 1. Verify email code in DB
    let res = state.auth_app.verify_email(payload).await?;

    // 2. Create dual-token session in Redis
    let session = session_management::create_session(
        &state.redis_pool,
        res.user_id,
        res.is_barber_profile_active
            .then(|| "BARBER")
            .unwrap_or("CLIENT"),
        &state.config.jwt_secret,
    )
    .await?;

    // 3. Generate Set-Cookie header for the refresh token
    let headers = auth_cookies::create_refresh_cookie(&session.refresh_token)?;

    // 4. Return both the cookie header AND the JSON response (access_token + user data)
    Ok((
        headers,
        Json(ApiResponse::success(AuthResponse {
            access_token: session.access_token,
            token_type: session.token_type,
            user: session.user,
        })),
    ))
}

pub async fn resend_verification(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, ApiError> {
    let res = state.auth_app.resend_verification(payload).await?;
    Ok(Json(ApiResponse::success(res)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<ApiResponse<AuthResponse>>), ApiError> {
    println!("hited from app");
    let res = state.auth_app.login(payload).await?;
    // 2. Create dual-token session in Redis
    let session = session_management::create_session(
        &state.redis_pool,
        res.user_id,
        res.is_barber_profile_active
            .then(|| "BARBER")
            .unwrap_or("CLIENT"),
        &state.config.jwt_secret,
    )
    .await?;

    // 3. Generate Set-Cookie header for the refresh token
    let headers = auth_cookies::create_refresh_cookie(&session.refresh_token)?;

    // 4. Return both the cookie header AND the JSON response (access_token + user data)
    Ok((
        headers,
        Json(ApiResponse::success(AuthResponse {
            access_token: session.access_token,
            token_type: session.token_type,
            user: session.user,
        })),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthContext,
    refresh_token: RefreshToken,
) -> Result<(HeaderMap, Json<&'static str>), ApiError> {
    // 1. Blacklist Access Token and delete Refresh Token in Redis
    session_management::terminate_session(
        &state.redis_pool,
        auth.user_id(),
        auth.jti(),
        auth.exp(),
        Some(refresh_token.as_str()),
    )
    .await?;
    //2. call the app to log that the user has logged out
    //state.auth_app.logout(auth.user_id()).await?;
    // 2. Build the HeaderMap that instructs the browser to clear the cookie
    let headers = auth_cookies::clear_refresh_cookie();

    // 3. Return (HeaderMap, Json) tuple - Axum converts both into the HTTP response
    Ok((headers, Json("Successfully logged out")))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    refresh_token: RefreshToken,
) -> Result<(HeaderMap, Json<ApiResponse<AuthResponse>>), ApiError> {
    // 1. Rotate tokens
    println!(
        "Rotating session for refresh token: {}",
        refresh_token.as_str()
    );
    let session: Session = session_management::rotate_session(
        &state.redis_pool,
        refresh_token.as_str(),
        &state.config.jwt_secret,
    )
    .await?;
    println!(
        "Rotating session for refresh token: {}",
        session.refresh_token
    );

    // 2. Build new Set-Cookie header for rotated refresh token
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
