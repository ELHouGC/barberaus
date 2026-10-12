use crate::error::api_error::ApiError;
use crate::transport::response::ApiResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, user_id) = match self {
            ApiError::MissingToken => (
                StatusCode::OK,
                "MISSING_TOKEN",
                "Authentication token is missing.",
                None,
            ),
            ApiError::InvalidToken => (
                StatusCode::OK,
                "INVALID_TOKEN",
                "Supplied authentication token is invalid.",
                None,
            ),
            ApiError::TokenExpired => (
                StatusCode::OK,
                "TOKEN_EXPIRED",
                "Your session has expired.",
                None,
            ),
            ApiError::InvalidCredentials => (
                StatusCode::OK,
                "INVALID_CREDENTIALS",
                "Invalid email or password.",
                None,
            ),
            ApiError::Forbidden => (
                StatusCode::OK,
                "FORBIDDEN",
                "You are not allowed to perform this action.",
                None,
            ),
            ApiError::TenantContextRequired => (
                StatusCode::OK,
                "TENANT_CONTEXT_REQUIRED",
                "Tenant context required for this operation.",
                None,
            ),
            ApiError::MembershipNotFound => (
                StatusCode::OK,
                "MEMBERSHIP_NOT_FOUND",
                "User does not have an active membership in target tenant.",
                None,
            ),
            ApiError::EmailAlreadyExists => (
                StatusCode::OK,
                "EMAIL_EXISTS",
                "User with this email address already exists.",
                None,
            ),
            ApiError::InvalidVerificationToken => (
                StatusCode::OK,
                "INVALID_VERIFICATION_TOKEN",
                "Invalid verification token.",
                None,
            ),
            ApiError::VerificationTokenExpired => (
                StatusCode::OK,
                "VERIFICATION_TOKEN_EXPIRED",
                "Verification token has expired.",
                None,
            ),
            ApiError::VerificationTokenNotFound => (
                StatusCode::OK,
                "VERIFICATION_TOKEN_NOT_FOUND",
                "Verification token not found.",
                None,
            ),
            ApiError::AccountAlreadyVerified => (
                StatusCode::OK,
                "ACCOUNT_ALREADY_VERIFIED",
                "Account is already verified.",
                None,
            ),
            ApiError::AccountNotVerified { user_id } => (
                StatusCode::OK,
                "ACCOUNT_NOT_VERIFIED",
                "Account is not verified. Please verify your email.",
                Some(user_id),
            ),
            ApiError::UserNotFound => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "USER_NOT_FOUND",
                "User not found.",
                None,
            ),

            // Input validation errors return OK status with structured code
            ApiError::InvalidInput(ref msg) => {
                (StatusCode::OK, "INVALID_INPUT", msg.as_str(), None)
            }

            // Server-side unexpected failures remain 500s
            ApiError::DatabaseError
            | ApiError::Unexpected
            | ApiError::InternalServerError
            | ApiError::EmailSendingFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An unexpected internal error occurred.",
                None,
            ),
       
        };

        let response_body: ApiResponse<()> = ApiResponse::error(code, message, user_id);
        (status, Json(response_body)).into_response()
    }
}
