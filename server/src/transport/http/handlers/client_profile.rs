use crate::{
    AppState,
    application::error::AppError,
    transport::{
        dto::client_profile::{
            FollowBarberRequest, SignatureResponse, StylePreferencesResponse, UpdateAvatarRequest,
            UpdateStylePreferencesRequest,
        },
        response::ApiResponse,
    },
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
};

use crate::transport::http::extractors::auth_context::AuthContext;

pub async fn handle_update_avatar_command(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<UpdateAvatarRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let avatar_payload = payload.validate(
        &state.client_profile_app.cloudinary.cloud_name,
        auth.claims.sub,
    )?;

    state
        .client_profile_app
        .update_avatar(avatar_payload)
        .await?;
    Ok(Json(ApiResponse::success(())))
}

pub async fn handle_get_upload_signature(
    State(app_state): State<AppState>,
    auth: AuthContext, // or your auth extractor
) -> Result<Json<ApiResponse<SignatureResponse>>, AppError> {
    let signature = app_state
        .client_profile_app
        .generate_avatar_signature(auth.claims.sub)
        .await?;
    Ok(Json(ApiResponse::success(signature)))
}

pub async fn handle_append_style_preferences(
    State(state): State<AppState>,
    auth: AuthContext,
    payload: Result<Json<UpdateStylePreferencesRequest>, JsonRejection>,
) -> Result<Json<ApiResponse<StylePreferencesResponse>>, AppError> {
    let Json(payload) = payload.map_err(|_| {
        AppError::Validation(vec![crate::application::error::Violation {
            pointer: "/selected_style_photo_ids".to_string(),
            key: "validation.invalid_json",
        }])
    })?;
    let selected_photo_ids = payload.selected_photo_ids()?;

    let preferences = state
        .client_profile_app
        .append_style_preferences(auth.user_id(), selected_photo_ids)
        .await?;

    Ok(Json(ApiResponse::success(preferences)))
}

pub async fn handle_follow_barber_command(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<FollowBarberRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state
        .client_profile_app
        .follow_barber(auth.claims.sub, payload.barber_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}
