use std::sync::Arc;
use uuid::Uuid;

use crate::{
    cqrs::UserCqrsFramework, domain::{
        repositories::auth_repository::{UserAuthRepository, UserCredentialsRecord},
        services::{
            email_verification::EmailVerificationService,
            password_hasher_service::PasswordHasherService,
        },
        user::account_aggregate::UserCommand,
    }, error::ApiError,  transport::dto::auth::{
        LoginRequest, MessageResponse, ResendVerificationRequest, SignupRequest, SignupResponse, UserPayload, VerifyEmailRequest, 
    },
};

#[derive(Clone)]
pub struct AuthApp {
    pub auth_repo: Arc<dyn UserAuthRepository>,
    pub password_hasher: Arc<dyn PasswordHasherService>,
    pub user_cqrs_framework: Arc<UserCqrsFramework>,
    pub email_verification_service: Arc<EmailVerificationService>,

}

impl AuthApp {
    pub fn new(
        auth_repo: Arc<dyn UserAuthRepository>,
        password_hasher: Arc<dyn PasswordHasherService>,
        user_cqrs_framework: Arc<UserCqrsFramework>,
        email_verification_service: Arc<EmailVerificationService>,
      
    ) -> Self {
        Self {
            auth_repo,
            password_hasher,
            user_cqrs_framework,
            email_verification_service,
     
        }
    }

    pub async fn signup(&self, payload: SignupRequest) ->
     Result<SignupResponse, ApiError> {
        let password_hash = self.password_hasher.hash(payload.password).await?;
        let user_id = Uuid::new_v4();

        // Step 1: Create relational credentials record (inactive by default)
        let record = UserCredentialsRecord {
            user_id,
            email: payload.email.clone(),
            password_hash,
            full_name: payload.full_name.clone(),
            is_account_active: false,
            is_barber_profile_active: false,
        };

        self.auth_repo.create_user(record).await?;

        // Step 2: Dispatch CQRS event to stream
        let command_result = self
            .user_cqrs_framework
            .execute(
                &user_id.to_string(),
                UserCommand::RegisterUser {
                    user_id,
                    email: payload.email.clone(),
                    full_name: payload.full_name,
                },
            )
            .await;

        // Step 3: Rollback relational record if EventStore fails
        if let Err(err) = command_result {
            tracing::error!("CQRS command failed for user {user_id}, rolling back: {err:?}");
            let _ = self.auth_repo.delete_user(user_id).await;
            return Err(ApiError::InternalServerError);
        }

        // Step 4: Generate verification code and dispatch email
        self.email_verification_service
            .prepare_for_verification(user_id, &payload.email)
            .await?;

        Ok(SignupResponse {
            message: "Registration successful. Please check your email for your verification code."
                .to_string(),
            user_id,
        })
    }

    pub async fn verify_email(
        &self,
        payload: VerifyEmailRequest,
    ) -> Result<UserPayload, ApiError> {
        // 1. Verify code and activate account in DB
        self.email_verification_service
            .verify(payload.user_id, &payload.code)
            .await?;

        // 2. Fetch active user record to populate token claims & state
        let user = self
            .auth_repo
            .find_by_id(payload.user_id)
            .await?
            .ok_or(ApiError::UserNotFound)?;
        // todo  : the cqrs must be called here to active the account state and prepar it for onboarding process 
        Ok(
            UserPayload {
                user_id: user.user_id,
                is_barber_profile_active: user.is_barber_profile_active,        
            },
        )
    }

    pub async fn resend_verification(
        &self,
        payload: ResendVerificationRequest,
    ) -> Result<MessageResponse, ApiError> {
        self.email_verification_service
            .prepare_for_verification(payload.user_id, &payload.email)
            .await?;

        Ok(MessageResponse {
            message: "Verification code resent successfully.".to_string(),
        })
    }

    pub async fn login(&self, payload: LoginRequest) -> Result<UserPayload, ApiError> {
        let user = self
            .auth_repo
            .find_by_email(&payload.email)
            .await?
            .ok_or(ApiError::InvalidCredentials)?;

        let is_valid = self
            .password_hasher
            .verify(payload.password, user.password_hash)
            .await?;

        if !is_valid {
            return Err(ApiError::InvalidCredentials);
        }

        // Guard: Account activation check
        if !user.is_account_active {
            self.email_verification_service
                .handle_unverified_login_attempt(user.user_id, &user.email)
                .await?;

            return Err(ApiError::AccountNotVerified {
                user_id: user.user_id,
            });
        }
         
    

        Ok(UserPayload {
            user_id: user.user_id,
            is_barber_profile_active: user.is_barber_profile_active,
        })
    }
}
