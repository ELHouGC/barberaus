use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cqrs_es::{Aggregate, event_sink};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::DomainError;

// -----------------------------------------------------------------------------
// Aggregate State
// -----------------------------------------------------------------------------
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserAccountAggregate {
    pub user_id: Option<Uuid>,
    pub email: String,
    pub full_name: String,
    pub password_hash: String,
    pub is_email_verified: bool,
    pub has_client_profile: bool,
    pub has_barber_profile: bool,
}

// -----------------------------------------------------------------------------
// Commands
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAccountCommand {
    RegisterUser {
        user_id: Uuid,
        email: String,
        full_name: String,
        password_hash: String,
    },
    VerifyEmail,
    EnableClientProfile,
    EnableBarberProfile,
}

// -----------------------------------------------------------------------------
// Events
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserAccountEvent {
    UserRegistered {
        user_id: Uuid,
        email: String,
        full_name: String,
        password_hash: String,
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

fn default_registered_at() -> DateTime<Utc> {
    DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now)
}

// -----------------------------------------------------------------------------
// Aggregate Logic Implementation
// -----------------------------------------------------------------------------
#[async_trait]
impl Aggregate for UserAccountAggregate {
    type Command = UserAccountCommand;
    type Event = UserAccountEvent;
    type Error = DomainError;
    type Services = ();

    const TYPE: &'static str = "userAccount";

    async fn handle(
        &mut self,
        command: Self::Command,
        _services: &Self::Services,
        sink: &EventSink<Self>,
    ) -> Result<(), Self::Error> {
        match command {
            UserAccountCommand::RegisterUser {
                user_id,
                email,
                full_name,
                password_hash,
            } => {
                if self.user_id.is_some() {
                    return Err(DomainError::AlreadyExists);
                }
                if email.trim().is_empty() {
                    return Err(DomainError::InvalidEmail);
                }

                sink.write(
                    UserAccountEvent::UserRegistered {
                        user_id,
                        email,
                        full_name,
                        password_hash,
                        registered_at: Utc::now(),
                    },
                    self,
                )
                .await;
            }

            UserAccountCommand::VerifyEmail => {
                let user_id = self.user_id.ok_or(DomainError::UserNotFound)?;
                if self.is_email_verified {
                    return Err(DomainError::AlreadyVerified);
                }

                sink.write(
                    UserAccountEvent::EmailVerified {
                        user_id,
                        verified_at: Utc::now(),
                    },
                    self,
                )
                .await;
            }

            UserAccountCommand::EnableClientProfile => {
                let user_id = self.user_id.ok_or(DomainError::UserNotFound)?;
                if !self.is_email_verified {
                    return Err(DomainError::UserNotVerified);
                }
                if self.has_client_profile {
                    return Err(DomainError::ProfileAlreadyExists);
                }

                sink.write(
                    UserAccountEvent::ClientProfileEnabled {
                        user_id,
                        enabled_at: Utc::now(),
                    },
                    self,
                )
                .await;
            }

            UserAccountCommand::EnableBarberProfile => {
                let user_id = self.user_id.ok_or(DomainError::UserNotFound)?;
                if !self.is_email_verified {
                    return Err(DomainError::UserNotVerified);
                }
                if self.has_barber_profile {
                    return Err(DomainError::ProfileAlreadyExists);
                }

                sink.write(
                    UserAccountEvent::BarberProfileEnabled {
                        user_id,
                        enabled_at: Utc::now(),
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
            UserAccountEvent::UserRegistered {
                user_id,
                email,
                full_name,
                password_hash,
                ..
            } => {
                self.user_id = Some(user_id);
                self.email = email;
                self.full_name = full_name;
                self.password_hash = password_hash;
                self.is_email_verified = false;
                self.has_client_profile = false;
                self.has_barber_profile = false;
            }

            UserAccountEvent::EmailVerified { .. } => {
                self.is_email_verified = true;
            }

            UserAccountEvent::ClientProfileEnabled { .. } => {
                self.has_client_profile = true;
            }

            UserAccountEvent::BarberProfileEnabled { .. } => {
                self.has_barber_profile = true;
            }
        }
    }
}

use cqrs_es::DomainEvent;

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