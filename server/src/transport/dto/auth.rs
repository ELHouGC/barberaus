use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::error::{AppError, Violation},
    domain::user::account_aggregate::Gender,
};

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
}

impl SignupRequest {
    /// Validates basic constraints, accumulating or returning structural violations.
    pub fn validate(&self) -> Result<(), AppError> {
        let mut errors = Vec::new();

        if self.full_name.trim().is_empty() {
            errors.push(Violation {
                pointer: "/full_name".to_string(),
                key: "validation.required", // aligns with your i18n pattern
            });
        }

        let now = Utc::now().date_naive();
        if self.date_of_birth >= now {
            errors.push(Violation {
                pointer: "/date_of_birth".to_string(),
                key: "validation.date_of_birth_in_future",
            });
        } else {
            // Calculate age only if date of birth is validly in the past
            let age = now.years_since(self.date_of_birth).unwrap_or(0);
            if age < 13 {
                errors.push(Violation {
                    pointer: "/date_of_birth".to_string(),
                    key: "validation.underage",
                });
            }
        }

        if !errors.is_empty() {
            return Err(AppError::Validation(errors));
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub message: String,
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub user_id: Uuid,
    pub code: String,
}

#[derive(Deserialize)]
pub struct ResendVerificationRequest {
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub user: UserPayload,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserPayload,
}

#[derive(Serialize)]
pub struct UserPayload {
    pub user_id: Uuid,
    pub is_barber_profile_active: bool,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}
