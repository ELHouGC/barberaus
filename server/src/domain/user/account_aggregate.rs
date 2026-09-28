
use chrono::{ Utc};
use cqrs_es::{Aggregate, event_sink::EventSink};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::user::events::UserAccountEvent;
use crate::domain::errors::UserDomainError;

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
    },
    VerifyEmail,
    EnableClientProfile,
    EnableBarberProfile,
}




// -----------------------------------------------------------------------------
// Aggregate Logic Implementation
// -----------------------------------------------------------------------------

impl Aggregate for UserAccountAggregate {
    type Command = UserAccountCommand;
    type Event = UserAccountEvent;
    type Error = UserDomainError;
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
            } => {
                if self.user_id.is_some() {
                    return Err(UserDomainError::AlreadyExists);
                }
                if email.trim().is_empty() {
                    return Err(UserDomainError::InvalidEmail);
                }

                sink.write(
                    UserAccountEvent::UserRegistered {
                        user_id,
                        email,
                        full_name,
                        registered_at: Utc::now(),
                    },
                    self,
                )
                .await;
            }

            UserAccountCommand::VerifyEmail => {
                let user_id = self.user_id.ok_or(UserDomainError::UserNotFound)?;
                if self.is_email_verified {
                    return Err(UserDomainError::AlreadyVerified);
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
                let user_id = self.user_id.ok_or(UserDomainError::UserNotFound)?;
                if !self.is_email_verified {
                    return Err(UserDomainError::UserNotVerified);
                }
                if self.has_client_profile {
                    return Err(UserDomainError::ProfileAlreadyExists);
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
                let user_id = self.user_id.ok_or(UserDomainError::UserNotFound)?;
                if !self.is_email_verified {
                    return Err(UserDomainError::UserNotVerified);
                }
                if self.has_barber_profile {
                    return Err(UserDomainError::ProfileAlreadyExists);
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
                ..
            } => {
                self.user_id = Some(user_id);
                self.email = email;
                self.full_name = full_name;
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
