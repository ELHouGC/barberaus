use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::{
    state::AppState,
    transport::{
        http::handlers::client_profile::{
            handle_append_style_preferences, handle_get_upload_signature,
            handle_update_avatar_command,
        },
        middlewares::auth::require_auth,
    },
};

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route(
            "/api/v1/client/avatar/signature",
            get(handle_get_upload_signature),
        )
        .route("/api/v1/client/avatar", post(handle_update_avatar_command))
        .route(
            "/api/v1/me/style-preferences",
            put(handle_append_style_preferences),
        )
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_auth,
        ))
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use axum::{
        Json,
        body::Body,
        extract::State,
        http::{Request, StatusCode},
    };
    use deadpool_redis::{Config as RedisConfig, Runtime};
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        application::{
            auth_app::AuthApp,
            services::email_verification::{EmailSender, EmailVerificationService},
        },
        config::Config,
        cqrs::create_cqrs_framework,
        infrastructure::{
            argon2_password_hasher::Argon2PasswordHasher,
            email_sender::ConsoleEmailSender,
            jwt::Claims,
            persistence::{
                pg_auth_repository::PgUserAuthRepository,
                pg_email_verification_repository::PostgresEmailVerificationRepo,
            },
        },
        state::AppState,
        transport::{
            dto::client_profile::UpdateStylePreferencesRequest,
            http::{
                handlers::client_profile::handle_append_style_preferences,
                router::client_profile::create_router,
                 extractors::auth_context::AuthContext
            }
        },
    };

    fn test_state() -> AppState {
        let db_pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/barberos_test")
            .unwrap();
        let redis_pool = RedisConfig::from_url("redis://127.0.0.1")
            .create_pool(Some(Runtime::Tokio1))
            .unwrap();

        let (user_cqrs, client_cqrs, client_view_repo, user_view_repo) =
            create_cqrs_framework(db_pool.clone());
        let email_sender: Arc<dyn EmailSender> = Arc::new(ConsoleEmailSender);
        let email_verification = Arc::new(EmailVerificationService::new(
            Arc::new(PostgresEmailVerificationRepo::new(db_pool.clone())),
            email_sender,
            "noreply@example.com".to_string(),
        ));
        let auth_app = AuthApp::new(
            Arc::new(PgUserAuthRepository::new(db_pool)),
            Arc::new(Argon2PasswordHasher),
            Arc::new(user_cqrs),
            email_verification,
        );
        let client_profile_app = Arc::new(crate::application::client_profile_app::ClientProfileApp::new(
            client_cqrs,
            Arc::new(crate::application::services::cloudinary_service::CloudinaryService::from_credentials(
                "test-cloud".to_string(),
                "test-key".to_string(),
                "test-secret".to_string(),
            )),
            client_view_repo,
        ));

        AppState {
            auth_app,
            client_profile_app,
            user_view_repo,
            redis_pool,
            config: Config {
                jwt_secret: "test-secret".to_string(),
                environment: crate::config::AppEnvironment::Development,
            },
        }
    }

    #[tokio::test]
    async fn style_preferences_route_rejects_unauthenticated_requests() {
        let response = create_router(test_state())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/me/style-preferences")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"selected_style_photo_ids":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers()["content-type"],
            "application/problem+json"
        );
    }

    #[tokio::test]
    async fn unknown_photo_id_is_rejected_before_command_execution() {
        let result = handle_append_style_preferences(
            State(test_state()),
            AuthContext::new(Claims {
                sub: uuid::Uuid::new_v4(),
                jti: "test-jti".to_string(),
                active_profile: "CLIENT".to_string(),
                exp: 0,
            }),
            Ok(Json(UpdateStylePreferencesRequest {
                selected_style_photo_ids: Some(vec!["unknown-photo".to_string()]),
            })),
        )
        .await;

        assert!(matches!(
            result,
            Err(crate::application::error::AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn duplicate_photo_ids_are_rejected_before_command_execution() {
        let result = handle_append_style_preferences(
            State(test_state()),
            AuthContext::new(Claims {
                sub: uuid::Uuid::new_v4(),
                jti: "test-jti".to_string(),
                active_profile: "CLIENT".to_string(),
                exp: 0,
            }),
            Ok(Json(UpdateStylePreferencesRequest {
                selected_style_photo_ids: Some(vec![
                    "gents-short-crop".to_string(),
                    "gents-short-crop".to_string(),
                ]),
            })),
        )
        .await;

        assert!(matches!(
            result,
            Err(crate::application::error::AppError::Validation(_))
        ));
    }
}
