use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    application::{
        auth_app::AuthApp,
        client_profile_app::ClientProfileApp,
        services::{
            cloudinary_service::CloudinaryService,
            email_verification::{EmailSender, EmailVerificationService},
        },
    }, config::{AppEnvironment, Config}, cqrs::{ create_cqrs_framework}, infrastructure::{
        argon2_password_hasher::Argon2PasswordHasher,
        email_sender::{ConsoleEmailSender, ResendEmailSender},
        persistence::{
            pg_auth_repository::PgUserAuthRepository,
            pg_email_verification_repository::PostgresEmailVerificationRepo,
        },
    },
};

#[derive(Clone)]
pub struct AppState {
    pub auth_app: AuthApp,
    pub client_profile_app: Arc<ClientProfileApp>,
    pub redis_pool: RedisPool,
    pub config: Config,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_pool: RedisPool, config: Config) -> Self {
        let auth_repo = Arc::new(PgUserAuthRepository::new(db_pool.clone()));

        let email_sender: Arc<dyn EmailSender> = match config.environment {
            AppEnvironment::Development => {
                Arc::new(ConsoleEmailSender)
            }
            AppEnvironment::Production => {
                let resend_api_key = std::env::var("RESEND_API_KEY")
                    .expect("RESEND_API_KEY must be set in production");

                Arc::new(ResendEmailSender::new(&resend_api_key))
            }
        };

        let verification_repo = Arc::new(PostgresEmailVerificationRepo::new(db_pool.clone()));
        let verification_service = Arc::new(EmailVerificationService::new(
            verification_repo,
            email_sender,
            std::env::var("FROM_EMAIL").expect("FROM_EMAIL must be set")),
        );

        let (user_cqrs, client_profile_cqrs_framework, client_profile_view_repo, _user_view_repo) =
            create_cqrs_framework(db_pool);
        let user_cqrs = Arc::new(user_cqrs);
        let password_hasher = Arc::new(Argon2PasswordHasher);

        let auth_app = AuthApp::new(auth_repo, password_hasher, user_cqrs, verification_service);
        let cloudinary_service =
            Arc::new(CloudinaryService::new().expect("Failed to initialize Cloudinary service"));

        let client_profile_app = Arc::new(ClientProfileApp::new(
            client_profile_cqrs_framework,
            cloudinary_service,
            client_profile_view_repo,
        ));

        Self {
            auth_app,
            client_profile_app,
            redis_pool,
            config,
        }
    }
}
