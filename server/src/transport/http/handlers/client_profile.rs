
use axum::{
    extract::{State, Json},
};
use crate::{
    AppState, application::error::AppError, transport::{
        dto::client_profile::{SignatureResponse, UpdateAvatarRequest}, response::ApiResponse
    }};

use crate::transport::middlewares::auth_context::AuthContext;

pub async fn handle_update_avatar_command(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<UpdateAvatarRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let avatar_payload = payload.validate(&state.client_profile_app.cloudinary.cloud_name , auth.claims.sub)?;

    state.client_profile_app.update_avatar(avatar_payload).await?;
    Ok(Json(ApiResponse::success(())))
}

pub async fn handle_get_upload_signature(
    State(app_state): State<AppState>,
    auth: AuthContext, // or your auth extractor
) -> Result<Json<SignatureResponse>, AppError> {
    let signature = app_state.client_profile_app.generate_avatar_signature(auth.claims.sub).await?;
    Ok(Json(signature))
}