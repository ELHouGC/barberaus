
use chrono::{ NaiveDate, Utc};
use cqrs_es::{Aggregate, event_sink::EventSink};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::user::events::UserAccountEvent;
use crate::domain::errors::UserDomainError;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    PreferNotToSay,
}


// -----------------------------------------------------------------------------
// Aggregate State
// -----------------------------------------------------------------------------
#[derive(Debug, Default, Clone, Serialize, Deserialize)]

pub struct UserAccountAggregate {
    pub user_id: Option<Uuid>,
    pub email: String,
    pub full_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<Gender>,
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
        date_of_birth: NaiveDate,
        gender: Gender,
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
                date_of_birth,
                gender,
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
                        date_of_birth: Some(date_of_birth),
                        gender: Some(gender),
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
                date_of_birth,
                gender,
                ..
            } => {
                self.user_id = Some(user_id);
                self.email = email;
                self.full_name = full_name;
                self.date_of_birth = date_of_birth;
                self.gender = gender;
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
