use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    domain::user::{account_aggregate::Gender, client_profile_aggregate::ClientProfileCommand},
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

// -----------------------------------------------------------------------------
//  Final Auth Stage .
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteOnboardingRequest {
    pub city: String,
    pub search_radius_km: Option<i32>,
    pub selected_styles: Vec<String>,
    #[serde(default)]
    pub followed_barber_ids: Vec<Uuid>,
}

impl CompleteOnboardingRequest {
    /// Validates input prior to database or aggregate execution
    pub fn validate(&self, user_id: Uuid) -> Result<(), ApiError> {
        if self.city.trim().is_empty() {
            return Err(ApiError::InvalidInput("City is required.".to_string()));
        }

        if let Some(radius) = self.search_radius_km {
            if !(1..=500).contains(&radius) {
                return Err(ApiError::InvalidInput(
                    "Search radius must be between 1 and 500 km.".to_string(),
                ));
            }
        }

        if self.selected_styles.is_empty() {
            return Err(ApiError::InvalidInput(
                "At least one haircut style must be selected.".to_string(),
            ));
        }

        if self.followed_barber_ids.contains(&user_id) {
            return Err(ApiError::InvalidInput(
                "You cannot follow your own profile.".to_string(),
            ));
        }

        Ok(())
    }

    pub fn into_command(self) -> ClientProfileCommand {
        ClientProfileCommand::CompleteOnboarding {
            city: self.city.trim().to_string(),
            search_radius_km: self.search_radius_km.unwrap_or(10),
            selected_styles: self.selected_styles,
            followed_barber_ids: self.followed_barber_ids,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResponse {
    pub client_id: Uuid,
    pub message: String,
}
