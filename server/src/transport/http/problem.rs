// transport/http/problem.rs
use axum::{http::{header, HeaderValue, StatusCode as S}, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use uuid::Uuid;
use crate::{application::error::*, domain::errors::{BarberDomainError, UserDomainError}};
use crate::domain::errors::ClientDomainError;

struct Spec { 
    status: S, 
    code: &'static str, 
    title: &'static str 
}
const fn spec(status: S, code: &'static str, title: &'static str) -> Spec { 
    Spec { 
        status, 
        code, 
        title 
    } 
}

fn classify(err: &AppError) -> Spec {
    match err {
        AppError::Auth(a) => match a {
            AuthError::MissingToken      => spec(S::UNAUTHORIZED, "MISSING_TOKEN", "Authentication required"),
            AuthError::InvalidToken      => spec(S::UNAUTHORIZED, "INVALID_TOKEN", "Invalid authentication token"),
            AuthError::TokenExpired      => spec(S::UNAUTHORIZED, "TOKEN_EXPIRED", "Session expired"),
            AuthError::InvalidCredentials=> spec(S::UNAUTHORIZED, "INVALID_CREDENTIALS", "Invalid email or password"),
            AuthError::AccountNotVerified{..} => spec(S::FORBIDDEN, "ACCOUNT_NOT_VERIFIED", "Account not verified"),
        },
        AppError::Access(a) => match a {
            AccessError::Forbidden             => spec(S::FORBIDDEN, "FORBIDDEN", "Action not allowed"),
            AccessError::TenantContextRequired => spec(S::BAD_REQUEST, "TENANT_CONTEXT_REQUIRED", "Tenant context required"),
            AccessError::MembershipNotFound    => spec(S::FORBIDDEN, "MEMBERSHIP_NOT_FOUND", "No active membership"),
        },
        AppError::Verification(v) => match v {
            VerificationError::InvalidToken => spec(S::BAD_REQUEST, "INVALID_VERIFICATION_TOKEN", "Invalid verification token"),
            VerificationError::Expired      => spec(S::BAD_REQUEST, "VERIFICATION_TOKEN_EXPIRED", "Verification token expired"),
        },
        AppError::Validation(_)         => spec(S::UNPROCESSABLE_ENTITY, "INVALID_INPUT", "Invalid input"),
        AppError::ClientProfile(d )      => client_profile(d),
        AppError::BarberProfile(d )      => barber_profile(d),

        AppError::User(d)               => user(d),
        AppError::NotFound(_)           => spec(S::NOT_FOUND, "USER_NOT_FOUND", "User not found"),
        AppError::Conflict(k) => match k {
            ConflictKind::EmailExists            => spec(S::CONFLICT, "EMAIL_EXISTS", "Email already registered"),
            ConflictKind::AccountAlreadyVerified => spec(S::CONFLICT, "ACCOUNT_ALREADY_VERIFIED", "Account already verified"),
        },
        AppError::ConcurrentModification => spec(S::CONFLICT, "CONCURRENT_MODIFICATION", "Concurrent modification"),
        AppError::Unavailable { .. }     => spec(S::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", "Service temporarily unavailable"),
        AppError::Internal(_)            => spec(S::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal error"),
    }
}

fn   client_profile(e: &ClientDomainError) -> Spec {
    use ClientDomainError::*;
    match e {
        AlreadyInitialized    => spec(S::CONFLICT, "CLIENT_PROFILE_ALREADY_INITIALIZED", "Client profile already initialized"),
        ProfileNotInitialized => spec(S::CONFLICT, "CLIENT_PROFILE_NOT_INITIALIZED", "Client profile not initialized"),
        AlreadyOnboarded      => spec(S::CONFLICT, "ONBOARDING_ALREADY_COMPLETED", "Onboarding already completed"),
        CannotFollowSelf      => spec(S::UNPROCESSABLE_ENTITY, "CANNOT_FOLLOW_SELF", "Cannot follow yourself"),
        AlreadyFollowing      => spec(S::CONFLICT, "ALREADY_FOLLOWING", "Already following"),
        InvalidAvatarUrl      => spec(S::UNPROCESSABLE_ENTITY, "INVALID_AVATAR_URL", "Invalid avatar URL"),
        InvalidState          => spec(S::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal error"),
    }
}
fn   barber_profile(e: &BarberDomainError) -> Spec {
    use BarberDomainError::*;
    match e {
        AlreadyInitialized    => spec(S::CONFLICT, "BARBER_PROFILE_ALREADY_INITIALIZED", "Barber profile already initialized"),
        ProfileNotInitialized => spec(S::CONFLICT, "BARBER_PROFILE_NOT_INITIALIZED", "Barber profile not initialized"),
        AlreadyOnboarded      => spec(S::CONFLICT, "ONBOARDING_ALREADY_COMPLETED", "Onboarding already completed"),
        CannotFollowSelf      => spec(S::UNPROCESSABLE_ENTITY, "CANNOT_FOLLOW_SELF", "Cannot follow yourself"),
        AlreadyFollowing      => spec(S::CONFLICT, "ALREADY_FOLLOWING", "Already following"),
        InvalidAvatarUrl      => spec(S::UNPROCESSABLE_ENTITY, "INVALID_AVATAR_URL", "Invalid avatar URL"),
        InvalidState          => spec(S::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal error"),
    }
}

fn user(e: &UserDomainError) -> Spec {
    use UserDomainError::*;
    match e {
        UserNotFound => spec(S::NOT_FOUND, "USER_NOT_FOUND", "User not found"),
        InvalidEmail => spec(S::UNPROCESSABLE_ENTITY, "INVALID_EMAIL", "Invalid email"),
        InvalidState => spec(S::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal error"),
        AlreadyExists => spec(S::CONFLICT, "USER_ALREADY_EXISTS", "User already exists"),
        AlreadyVerified => spec(S::CONFLICT, "USER_ALREADY_VERIFIED", "User already verified"),
        UserNotVerified => spec(S::FORBIDDEN, "USER_NOT_VERIFIED", "User not verified"),
        ProfileAlreadyExists => spec(S::CONFLICT, "PROFILE_ALREADY_EXISTS", "Profile already exists"),
        NotABarber => spec(S::UNPROCESSABLE_ENTITY, "NOT_A_BARBER", "Target profile is not a barber"),
    }
}

#[derive(Serialize)]
struct Problem {
    #[serde(rename = "type")] type_: String,
    title: &'static str,
    status: u16,
    code: &'static str,
    i18n_key: String,
    #[serde(skip_serializing_if = "Option::is_none")] user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Vec::is_empty")]   errors: Vec<Violation>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let Spec { status, code, title } = classify(&self);

        // Log once, here. Level follows the status class, so the policy lives in one rule.
        if status.is_server_error() { tracing::error!(code, error = ?self, "request failed"); }
        else                        { tracing::debug!(code, "request rejected"); }

        let (user_id, errors) = match self {
            AppError::Auth(AuthError::AccountNotVerified { user_id }) => (Some(user_id), vec![]),
            AppError::Validation(v) => (None, v),
            _ => (None, vec![]),
        };

        let slug = code.to_ascii_lowercase();
        let body = Problem {
            type_: format!("urn:barberapp:problem:{}", slug.replace('_', "-")),
            title, status: status.as_u16(), code,
            i18n_key: format!("errors.server.{slug}"), user_id, errors,
        };

        let mut res = (status, [(header::CONTENT_TYPE, "application/problem+json")], Json(body)).into_response();
        if status == S::UNAUTHORIZED {
            res.headers_mut().insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        res
    }
}