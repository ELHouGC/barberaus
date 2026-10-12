use thiserror::Error;
use uuid::Uuid;
#[allow(dead_code)]
#[derive(Debug, Error)]

pub enum ApiError {
    #[error("Authentication token is missing")]
    MissingToken,
    #[error("Supplied authentication token is invalid")]
    InvalidToken,
    #[error("Your session has expired")]
    TokenExpired,
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error("You are not allowed to perform this action")]
    Forbidden,
    #[error("Tenant context required for this operation")]
    TenantContextRequired,
    #[error("User does not have an active membership in target tenant")]
    MembershipNotFound,
    #[error("User with this email already exists")]
    EmailAlreadyExists,
    #[error("Database error occurred")]
    DatabaseError,
    #[error("An unexpected error occurred")]
    Unexpected,
    #[error("Internal server error occurred")]
    InternalServerError,
    #[error("Invalid verification token")]
    InvalidVerificationToken,
    #[error("Verification token has expired")]
    VerificationTokenExpired,
    #[error("Email sending failed")]
    EmailSendingFailed,
    #[error("Verification token not found")]
    VerificationTokenNotFound,
    #[error("Account is already verified")]
    AccountAlreadyVerified,
    #[error("Account is not verified")]
    AccountNotVerified { user_id: Uuid },
    #[error("User not found")]
    UserNotFound,
    // Input Validation Failure
    #[error("Validation error: {0}")]
    InvalidInput(String),
}
