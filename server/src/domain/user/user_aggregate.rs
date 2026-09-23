use cqrs_es::{Aggregate, event_sink::EventSink};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// 1. Domain Commands
#[derive(Debug, Serialize, Deserialize)]
pub enum UserCommand {
    RegisterUser {
        user_id: Uuid,
        email: String,
        full_name: String,
    },
}

// 2. Domain Events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserEvent {
    UserRegistered {
        user_id: Uuid,
        email: String,
        full_name: String,
    },
}

impl cqrs_es::DomainEvent for UserEvent {
    fn event_type(&self) -> String {
        match self {
            UserEvent::UserRegistered { .. } => "UserRegistered".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

// 3. Domain Errors
#[derive(Debug, Error)]
pub enum UserDomainError {
    #[error("User aggregate already initialized")]
    AlreadyRegistered,
}

// 4. Domain Aggregate State
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserAggregate {
    pub user_id: Option<Uuid>,
    pub email: String,
    pub full_name: String,
    pub is_active: bool,
}

impl Aggregate for UserAggregate {
    const TYPE: &'static str = "user";

    type Command = UserCommand;
    type Event = UserEvent;
    type Error = UserDomainError;
    type Services = ();

    async fn handle(
        &mut self,
        command: Self::Command,
        _services: &Self::Services,
        sink: &EventSink<Self>,
    ) -> Result<(), Self::Error> {
        match command {
            UserCommand::RegisterUser {
                user_id,
                email,
                full_name,
            } => {
                if self.user_id.is_some() {
                    return Err(UserDomainError::AlreadyRegistered);
                }

                sink.write(
                    UserEvent::UserRegistered {
                        user_id,
                        email,
                        full_name,
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
            UserEvent::UserRegistered {
                user_id,
                email,
                full_name,
            } => {
                self.user_id = Some(user_id);
                self.email = email;
                self.full_name = full_name;
                self.is_active = true;
            }
        }
    }
}
