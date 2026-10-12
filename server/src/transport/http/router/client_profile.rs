use axum::{Router, routing::get};


use crate::{

    state::AppState, transport::http::handlers::client_profile::{handle_get_upload_signature, handle_update_avatar_command},
};

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/api/client/avatar/signature", get(handle_get_upload_signature))
        .route("/api/client/avatar", get(handle_update_avatar_command))
        .with_state(app_state)
}

