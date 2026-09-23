use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
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
    pub refresh_token:String,
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