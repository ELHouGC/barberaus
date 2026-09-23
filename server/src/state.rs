use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    application::auth_app::AuthApp,
    config::Config,
    cqrs::create_cqrs_framework,
    domain::services::email_verification::{EmailSender, EmailVerificationService},
    infrastructure::persistence::{
        pg_auth_repository::PgUserAuthRepository,
        pg_email_verification_repository::PostgresEmailVerificationRepo,
    },
    infrastructure::{
        argon2_password_hasher::Argon2PasswordHasher,
        email_sender::{ConsoleEmailSender, ResendEmailSender},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub auth_app: AuthApp,
    pub redis_pool: RedisPool,
    pub config: Config,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_pool: RedisPool, config: Config) -> Self {
        let auth_repo = Arc::new(PgUserAuthRepository::new(db_pool.clone()));

        let email_sender: Arc<dyn EmailSender> =
            if std::env::var("ENV").unwrap_or_default() == "production" {
                let resend_api_key = std::env::var("RESEND_API_KEY")
                    .expect("RESEND_API_KEY must be set in production");
                Arc::new(ResendEmailSender::new(&resend_api_key))
            } else {
                Arc::new(ConsoleEmailSender::default())
            };

        let verification_repo = Arc::new(PostgresEmailVerificationRepo::new(db_pool.clone()));
        let verification_service = Arc::new(EmailVerificationService::new(
            verification_repo,
            email_sender,
            std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@barberos.com".to_string()),
        ));

        let user_cqrs = Arc::new(create_cqrs_framework(db_pool));
        let password_hasher = Arc::new(Argon2PasswordHasher);

        let auth_app = AuthApp::new(
            auth_repo,
            password_hasher,
            user_cqrs,
            verification_service,
        );

        Self {
            auth_app,
            redis_pool,
            config,
        }
    }
}
