use axum::{Router, middleware, routing::{get, post}};


use crate::{

    state::AppState, transport::{http::handlers::client_profile::{handle_get_upload_signature, handle_update_avatar_command}, middlewares::auth::require_auth},
};

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/api/v1/client/avatar/signature", get(handle_get_upload_signature))
        .route("/api/v1/client/avatar", post(handle_update_avatar_command))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_auth,
        ))
        .with_state(app_state)
}

