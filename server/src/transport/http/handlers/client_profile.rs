
use axum::{
    extract::{State, Json},
};
use crate::{
    AppState, error::ApiError, transport::{
        dto::client_profile::{SignatureResponse, UpdateAvatarRequest}, response::ApiResponse
    }};

use crate::transport::middlewares::auth_context::AuthContext;

pub async fn handle_update_avatar_command(
    State(state): State<AppState>,
    Json(payload): Json<UpdateAvatarRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let avatar_payload = payload.validate(&state.client_profile_app.cloudinary.cloud_name)?;

    state.client_profile_app.update_avatar(avatar_payload).await?;
    Ok(Json(ApiResponse::success(())))
}






pub async fn handle_get_upload_signature(
    State(app_state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<SignatureResponse>, ApiError> {
   
    let folder = format!("avatars/{}", auth.claims.sub.to_string());
    let signature_data = app_state.client_profile_app.cloudinary.generate_upload_signature(folder.as_str())
    .map_err(|_| ApiError::InternalServerError)?;
    Ok(Json(signature_data))
}
