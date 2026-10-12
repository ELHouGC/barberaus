use chrono::{DateTime, NaiveDate, Utc};
use cqrs_es::{EventEnvelope, View};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::user::account_aggregate::{Gender, UserAccountAggregate};
use crate::domain::user::client_profile_aggregate::ClientProfileAggregate;
use crate::domain::user::events::ClientProfileEvent;
use crate::domain::user::events::UserAccountEvent;

// -----------------------------------------------------------------------------
// Unified View Data Structure
// -----------------------------------------------------------------------------
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedUserView {
    pub user_id: Option<Uuid>,
    pub identity: Option<IdentityViewSection>,
    pub client_profile: Option<ClientProfileViewSection>,
    pub barber_profile: Option<BarberProfileViewSection>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentityViewSection {
    pub email: String,
    pub full_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<Gender>,
    pub is_email_verified: bool,
    pub has_client_profile: bool,
    pub has_barber_profile: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientProfileViewSection {
    pub is_initialized: bool,
    pub avatar_url: Option<String>,
    pub onboarding_completed: bool,
    pub city: Option<String>,
    pub search_radius_km: i32,
    pub selected_styles: Vec<String>,
    pub followed_barber_ids: Vec<Uuid>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BarberProfileViewSection {
    pub is_initialized: bool,
    pub business_name: Option<String>,
    pub city: Option<String>,
    pub follower_count: i32,
}

// -----------------------------------------------------------------------------
// 1. UserAccount Projection Logic
// -----------------------------------------------------------------------------
impl View<UserAccountAggregate> for UnifiedUserView {
    fn update(&mut self, event: &EventEnvelope<UserAccountAggregate>) {
        match &event.payload {
            UserAccountEvent::UserRegistered {
                user_id,
                email,
                full_name,
                registered_at,
                date_of_birth,
                gender,
                ..
            } => {
                self.user_id = Some(*user_id);
                self.identity = Some(IdentityViewSection {
                    email: email.clone(),
                    full_name: full_name.clone(),
                    date_of_birth: *date_of_birth,
                    gender: *gender,
                    is_email_verified: false,
                    has_client_profile: false,
                    has_barber_profile: false,
                });
                self.updated_at = Some(*registered_at);
            }

            UserAccountEvent::EmailVerified { verified_at, .. } => {
                if let Some(identity) = &mut self.identity {
                    identity.is_email_verified = true;
                }
                self.updated_at = Some(*verified_at);
            }

            UserAccountEvent::ClientProfileEnabled { enabled_at, .. } => {
                if let Some(identity) = &mut self.identity {
                    identity.has_client_profile = true;
                }
                self.updated_at = Some(*enabled_at);
            }

            UserAccountEvent::BarberProfileEnabled { enabled_at, .. } => {
                if let Some(identity) = &mut self.identity {
                    identity.has_barber_profile = true;
                }
                self.updated_at = Some(*enabled_at);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// 2. ClientProfile Projection Logic
// -----------------------------------------------------------------------------
impl View<ClientProfileAggregate> for UnifiedUserView {
    fn update(&mut self, event: &EventEnvelope<ClientProfileAggregate>) {
        match &event.payload {
            ClientProfileEvent::ClientProfileInitialized { created_at, .. } => {
                self.client_profile = Some(ClientProfileViewSection {
                    is_initialized: true,
                    avatar_url: None,
                    onboarding_completed: false,
                    city: None,
                    search_radius_km: 10, // Default search radius
                    selected_styles: Vec::new(),
                    followed_barber_ids: Vec::new(),
                });
                self.updated_at = Some(*created_at);
            }
            ClientProfileEvent::AvatarUpdated { avatar_url } => {
                if let Some(profile) = &mut self.client_profile {
                    profile.avatar_url = Some(avatar_url.clone());
                }
            }

            ClientProfileEvent::ClientOnboarded {
                city,
                search_radius_km,
                selected_styles,
                followed_barber_ids,
                onboarded_at,
                ..
            } => {
                if let Some(profile) = &mut self.client_profile {
                    profile.city = Some(city.clone());
                    profile.search_radius_km = *search_radius_km;
                    profile.selected_styles = selected_styles.clone();
                    profile.followed_barber_ids = followed_barber_ids.clone();
                    profile.onboarding_completed = true;
                }
                self.updated_at = Some(*onboarded_at);
            }

            ClientProfileEvent::BarberFollowed {
                barber_id,
                followed_at,
                ..
            } => {
                if let Some(profile) = &mut self.client_profile {
                    if !profile.followed_barber_ids.contains(barber_id) {
                        profile.followed_barber_ids.push(*barber_id);
                    }
                }
                self.updated_at = Some(*followed_at);
            }
        }
    }
}
