
use chrono:: Utc;
use cqrs_es::{Aggregate, DomainEvent, event_sink::EventSink};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::user::events::ClientProfileEvent;

use crate::domain::errors::ClientDomainError;

// -----------------------------------------------------------------------------
// Aggregate State
// -----------------------------------------------------------------------------
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClientProfileAggregate {
    pub client_id: Option<Uuid>,
    pub is_initialized: bool,
    pub onboarding_completed: bool,
    pub city: Option<String>,
    pub search_radius_km: i32,
    pub selected_styles: Vec<String>,
    pub followed_barber_ids: Vec<Uuid>,
}

// -----------------------------------------------------------------------------
// Commands
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientProfileCommand {
    InitializeProfile {
        client_id: Uuid,
    },
    CompleteOnboarding {
        city: String,
        search_radius_km: i32,
        selected_styles: Vec<String>,
        followed_barber_ids: Vec<Uuid>,
    },
    FollowBarber {
        barber_id: Uuid,
    },
}


// -----------------------------------------------------------------------------
// Aggregate Logic Implementation
// -----------------------------------------------------------------------------

impl Aggregate for ClientProfileAggregate {
    type Command = ClientProfileCommand;
    type Event = ClientProfileEvent;
    type Error = ClientDomainError;
    type Services = ();
const TYPE: &'static str = "ClientProfile";


    async fn handle(
        &mut self,
        command: Self::Command,
        _services: &Self::Services,
        sink: &EventSink<Self>,
    ) -> Result<(), Self::Error> {
        match command {
            ClientProfileCommand::InitializeProfile { client_id } => {
                if self.is_initialized {
                    return Err(ClientDomainError::AlreadyInitialized);
                }

                sink.write(
                    ClientProfileEvent::ClientProfileInitialized {
                        client_id,
                        created_at: Utc::now(),
                    },
                    self,
                )
                .await;
            }

            ClientProfileCommand::CompleteOnboarding {
                city,
                search_radius_km,
                selected_styles,
                followed_barber_ids,
            } => {
                if !self.is_initialized {
                    return Err(ClientDomainError::ProfileNotInitialized);
                }
                if self.onboarding_completed {
                    return Err(ClientDomainError::AlreadyOnboarded);
                }

                let client_id = self.client_id.ok_or(ClientDomainError::InvalidState)?;

                sink.write(
                    ClientProfileEvent::ClientOnboarded {
                        client_id,
                        city,
                        search_radius_km,
                        selected_styles,
                        followed_barber_ids,
                        onboarded_at: Utc::now(),
                    },
                    self,
                )
                .await;
            }

            ClientProfileCommand::FollowBarber { barber_id } => {
                if !self.is_initialized {
                    return Err(ClientDomainError::ProfileNotInitialized);
                }

                let client_id = self.client_id.ok_or(ClientDomainError::InvalidState)?;

                if client_id == barber_id {
                    return Err(ClientDomainError::CannotFollowSelf);
                }
                if self.followed_barber_ids.contains(&barber_id) {
                    return Err(ClientDomainError::AlreadyFollowing);
                }

                sink.write(
                    ClientProfileEvent::BarberFollowed {
                        client_id,
                        barber_id,
                        followed_at: Utc::now(),
                    },
                    self,
                )
                .await;
            }
        }

        Ok(())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            ClientProfileEvent::ClientProfileInitialized { client_id, .. } => {
                self.client_id = Some(client_id);
                self.is_initialized = true;
            }

            ClientProfileEvent::ClientOnboarded {
                city,
                search_radius_km,
                selected_styles,
                followed_barber_ids,
                ..
            } => {
                self.city = Some(city);
                self.search_radius_km = search_radius_km;
                self.selected_styles = selected_styles;
                self.followed_barber_ids = followed_barber_ids;
                self.onboarding_completed = true;
            }

            ClientProfileEvent::BarberFollowed { barber_id, .. } => {
                if !self.followed_barber_ids.contains(&barber_id) {
                    self.followed_barber_ids.push(barber_id);
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// DomainEvent Implementation
// -----------------------------------------------------------------------------
impl DomainEvent for ClientProfileEvent {
    fn event_type(&self) -> String {
        match self {
            ClientProfileEvent::ClientProfileInitialized { .. } => {
                "ClientProfileInitialized".to_string()
            }
            ClientProfileEvent::ClientOnboarded { .. } => "ClientOnboarded".to_string(),
            ClientProfileEvent::BarberFollowed { .. } => "BarberFollowed".to_string(),
        }
    }

    fn event_version(&self) -> String {
        match self {
            ClientProfileEvent::ClientProfileInitialized { .. } => "1.0".to_string(),
            ClientProfileEvent::ClientOnboarded { .. } => "1.0".to_string(),
            ClientProfileEvent::BarberFollowed { .. } => "1.0".to_string(),
        }
    }
}