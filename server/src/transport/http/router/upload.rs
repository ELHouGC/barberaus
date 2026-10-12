use axum::{Router, routing::get};
use std::sync::Arc;

use crate::{
    application::services::cloudinary_service::CloudinaryService,
    transport::http::handlers::cloudinary_upload::handle_get_upload_signature,
};

pub fn create_router(cloudinary: Arc<CloudinaryService>) -> Router {
    Router::new()
        .route("/api/uploads/signature", get(handle_get_upload_signature))
        .with_state(cloudinary)
}
