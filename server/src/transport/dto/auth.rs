use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::user::{account_aggregate::Gender},
    error::ApiError,
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
    /// Validates basic constraints, including legal age.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.full_name.trim().is_empty() {
            return Err(ApiError::InvalidInput("Full name is required".into()));
        }

        let now = Utc::now().date_naive();
        if self.date_of_birth >= now {
            return Err(ApiError::InvalidInput(
                "Date of birth must be in the past".into(),
            ));
        }

        // Calculate age
        let age = now.years_since(self.date_of_birth).unwrap_or(0);
        if age < 13 {
            // Set your enforcement boundary here (e.g., 13, 16, or 18)
            return Err(ApiError::InvalidInput(
                "User does not meet the minimum age requirement".into(),
            ));
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
    pub email: String,
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
