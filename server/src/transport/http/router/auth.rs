use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    state::AppState, transport::{
        http::auth_handlers::{login, logout, refresh_token, resend_verification, signup, verify_email}, middlewares::auth::require_auth,
    },
};

pub fn create_router(app_state: AppState) -> Router {
    // Unprotected Routes
    let public_routes = Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/session/refresh", post(refresh_token))
        .route("/auth/resend-verification", post(resend_verification));

    // Protected Routes (All routes inside get perimeter defense)
    let protected_routes = Router::new()
        .route("/auth/session/logout", post(logout))
        // .route("/profile", get(get_profile))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_auth,
        ));

    Router::new()
        .nest("/api/v1", public_routes)
        .nest("/api/v1", protected_routes)
        .with_state(app_state)
}