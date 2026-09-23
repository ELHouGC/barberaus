use crate::error::ApiError;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize , Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub jti: String,
    pub active_profile: String,
    pub exp: usize,
}

pub fn mint_token(user_id: Uuid, profile: &str, secret: &str) -> Result<String, ApiError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;
    let jti = Uuid::new_v4().to_string();
    println!(" jti: {}",  jti);
    let claims = Claims {
        sub: user_id,
        jti: jti,
        active_profile: profile.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| ApiError::Unexpected)
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ApiError::InvalidToken)
}
