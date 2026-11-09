// application/services/auth_app.rs
//
// STYLE RULES FOR THIS FILE
// -------------------------
// 1. Pure refactor: same flows, same order of operations. Only error handling and logging change.
// 2. Everything returns `AppError`. Errors from ports/services/CQRS are converted by `?`
//    (`From` impls live in `application/error.rs`), so there is no manual conversion code.
// 3. LOG EACH ERROR ONCE. `problem.rs` logs whatever we return. So here we only log:
//      - business events (`info!`),
//      - context that would otherwise be lost ("we rolled back"),
//      - errors we SWALLOW (a failed rollback): nobody else will ever see those.
// 4. Spans: `#[instrument(skip_all)]` because the payloads hold passwords/emails.
//    `user_id` is attached to the span, so every log line emitted inside a method
//    carries it automatically; no need to repeat it in each `info!`.
// 5. NEVER log emails, passwords, codes or tokens.

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{
        error::{AppError, AuthError, ConflictKind, Resource},
        ports::auth_repository::{UserAuthRepository, UserCredentialsRecord},
        services::{
            email_verification::EmailVerificationService,
            password_hasher_service::PasswordHasherService,
        },
    },
    cqrs::UserCqrsFramework,
    domain::user::account_aggregate::UserAccountCommand,
    transport::dto::auth::{
        LoginRequest, MessageResponse, ResendVerificationRequest, SignupRequest, SignupResponse,
        UserPayload, VerifyEmailRequest,
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

    // `Empty` declares the field now; we fill it with `record` once the id exists.
    #[tracing::instrument(skip_all, fields(user_id = tracing::field::Empty))]
    pub async fn signup(&self, payload: SignupRequest) -> Result<SignupResponse, AppError> {
        let password_hash = self.password_hasher.hash(payload.password).await?;
        let user_id = Uuid::new_v4();
        tracing::Span::current().record("user_id", tracing::field::display(user_id));

        // Step 1: Create relational credentials record
        let record = UserCredentialsRecord {
            user_id,
            email: payload.email.clone(),
            password_hash,
            full_name: payload.full_name.clone(),
            is_account_active: false,
            is_barber_profile_active: false,
        };

        self.auth_repo.create_user(record).await?;

        // Step 2: Dispatch CQRS event command to event store
        let command_result = self
            .user_cqrs_framework
            .execute(
                &user_id.to_string(),
                UserAccountCommand::RegisterUser {
                    user_id,
                    email: payload.email.clone(),
                    full_name: payload.full_name,
                    date_of_birth: payload.date_of_birth,
                    gender: payload.gender,
                },
            )
            .await;

        // Step 3: Roll back the relational record if the event store fails,
        // then propagate the ORIGINAL error.
        if let Err(err) = command_result {
            // Context only: the error itself is logged once by `problem.rs`.
            tracing::warn!("RegisterUser command failed, rolling back user credentials");

            // This error is SWALLOWED (we return the original one instead),
            // so this is the only place it can ever be logged.
            if let Err(rollback_err) = self.auth_repo.delete_user(user_id).await {
                tracing::error!(error = ?rollback_err, "CRITICAL: failed to roll back user credentials after CQRS failure");
            }

            // `.into()` uses `From<AggregateError<E>> for AppError`.
            return Err(err.into());
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

    #[tracing::instrument(skip_all, fields(user_id = %payload.user_id))]
    pub async fn verify_email(&self, payload: VerifyEmailRequest) -> Result<UserPayload, AppError> {
        // 1. Verify code and activate account in DB
        self.email_verification_service
            .verify(payload.user_id, &payload.code)
            .await?;

        // 2. Fetch active user record to populate token claims & state
        let user = self
            .auth_repo
            .find_by_id(payload.user_id)
            .await?
            .ok_or(AppError::NotFound(Resource::User))?;

        // todo: the cqrs must be called here to activate the account state and prepare it for onboarding

        // `?` converts `AggregateError<UserDomainError>` into `AppError` for us.
        // This replaces the manual `if let Err` + `InternalServerError`, which also
        // threw the real cause away.
        self.user_cqrs_framework
            .execute(&user.user_id.to_string(), UserAccountCommand::VerifyEmail)
            .await?;

        Ok(UserPayload {
            user_id: user.user_id,
            is_barber_profile_active: user.is_barber_profile_active,
        })
    }

    // SECURITY TODO (separate commit, see the guide): `payload.email` is attacker-controlled.
    // The code must be sent to the email STORED for `user_id`, never to one from the request.
    #[tracing::instrument(skip_all, fields(user_id = %payload.user_id))]
    pub async fn resend_verification(
        &self,
        payload: ResendVerificationRequest,
    ) -> Result<MessageResponse, AppError> {
        let user = self
            .auth_repo
            .find_by_id(payload.user_id)
            .await?
            .ok_or(AppError::NotFound(Resource::User))?;
        if user.is_account_active {
            return Err(AppError::Conflict(ConflictKind::AccountAlreadyVerified));
        }
        self.email_verification_service
            .prepare_for_verification(user.user_id, &user.email) // stored email
            .await?;

        Ok(MessageResponse {
            message: "Verification code resent successfully.".to_string(),
        })
    }

    #[tracing::instrument(skip_all, fields(user_id = tracing::field::Empty))]
    pub async fn login(&self, payload: LoginRequest) -> Result<UserPayload, AppError> {
        // Unknown email and wrong password give the SAME error on purpose,
        // so the API does not reveal which emails are registered.
        let user = self
            .auth_repo
            .find_by_email(&payload.email)
            .await?
            .ok_or(AppError::Auth(AuthError::InvalidCredentials))?;
        tracing::Span::current().record("user_id", tracing::field::display(user.user_id));

        let is_valid = self
            .password_hasher
            .verify(payload.password, user.password_hash)
            .await?;

        if !is_valid {
            // Replaces `print!("... {}", payload.email)`, which wrote the email to stdout.
            // The span already carries the user id: enough to spot brute-force attempts.
            tracing::info!("login rejected: wrong password");
            return Err(AppError::Auth(AuthError::InvalidCredentials));
        }

        // Guard: account activation check
        if !user.is_account_active {
            self.email_verification_service
                .handle_unverified_login_attempt(user.user_id, &user.email)
                .await?;

            return Err(AppError::Auth(AuthError::AccountNotVerified {
                user_id: user.user_id,
            }));
        }

        Ok(UserPayload {
            user_id: user.user_id,
            is_barber_profile_active: user.is_barber_profile_active,
        })
    }
}
