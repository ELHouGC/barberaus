use crate::application::services::cloudinary_service::CloudinaryService;
use crate::application::services::cloudinary_service::SignatureResponse;
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;

pub async fn handle_get_upload_signature(
    State(cloudinary): State<Arc<CloudinaryService>>,
    // claims: Claims, // Add your JWT auth extractor here to protect the endpoint
) -> Result<Json<SignatureResponse>, (StatusCode, String)> {
    let folder = "avatars";

    let signature_data = cloudinary.generate_upload_signature(folder).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to generate signature: {}", e),
        )
    })?;

    Ok(Json(signature_data))
}
