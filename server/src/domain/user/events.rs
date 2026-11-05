use chrono::{DateTime, NaiveDate, Utc};
use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::user::{account_aggregate::Gender, value_objects::CropCoordinates};
// -----------------------------------------------------------------------------
// User Events
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserAccountEvent {
    UserRegistered {
        user_id: Uuid,
        email: String,
        full_name: String,
        #[serde(default)]
        date_of_birth: Option<NaiveDate>,
        #[serde(default)]
        gender: Option<Gender>,
        #[serde(default = "default_registered_at")]
        registered_at: DateTime<Utc>,
    },
    EmailVerified {
        user_id: Uuid,
        verified_at: DateTime<Utc>,
    },
    ClientProfileEnabled {
        user_id: Uuid,
        enabled_at: DateTime<Utc>,
    },
    BarberProfileEnabled {
        user_id: Uuid,
        enabled_at: DateTime<Utc>,
    },
}

impl DomainEvent for UserAccountEvent {
    fn event_type(&self) -> String {
        match self {
            UserAccountEvent::UserRegistered { .. } => "UserRegistered".to_string(),
            UserAccountEvent::EmailVerified { .. } => "EmailVerified".to_string(),
            UserAccountEvent::ClientProfileEnabled { .. } => "ClientProfileEnabled".to_string(),
            UserAccountEvent::BarberProfileEnabled { .. } => "BarberProfileEnabled".to_string(),
        }
    }

    fn event_version(&self) -> String {
        match self {
            UserAccountEvent::UserRegistered { .. } => "1.0".to_string(),
            UserAccountEvent::EmailVerified { .. } => "1.0".to_string(),
            UserAccountEvent::ClientProfileEnabled { .. } => "1.0".to_string(),
            UserAccountEvent::BarberProfileEnabled { .. } => "1.0".to_string(),
        }
    }
}

// -----------------------------------------------------------------------------
// Client Events
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientProfileEvent {
    ClientProfileInitialized {
        client_id: Uuid,
        created_at: DateTime<Utc>,
    },
    AvatarUpdated { avatar_url: String , crop_coordinates: CropCoordinates }, // (x, y, width, height)
    StylePreferencesUpdated {
        selected_styles: Vec<String>,
        updated_at: DateTime<Utc>,
    },
    BarberFollowed {
        client_id: Uuid,
        barber_id: Uuid,
        followed_at: DateTime<Utc>,
    },
}

fn default_registered_at() -> DateTime<Utc> {
    DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now)
}
