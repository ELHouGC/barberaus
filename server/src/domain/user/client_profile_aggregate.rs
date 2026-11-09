use crate::domain::user::{events::ClientProfileEvent, value_objects::CropCoordinates};
use chrono::Utc;
use cqrs_es::{Aggregate, DomainEvent, event_sink::EventSink};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::domain::errors::ClientDomainError;

// -----------------------------------------------------------------------------
// Aggregate State
// -----------------------------------------------------------------------------
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClientProfileAggregate {
    pub client_id: Option<Uuid>,
    pub is_initialized: bool,
    pub avatar_url: Option<String>,
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
    UpdateAvatar {
        user_id: String,
        avatar_url: String,
        crop_coordinates: CropCoordinates,
    },
    AppendStylePreferences {
        selected_styles: Vec<String>,
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
            ClientProfileCommand::UpdateAvatar {
                avatar_url,
                crop_coordinates,
                ..
            } => {
                if self.is_initialized == false {
                    return Err(ClientDomainError::ProfileNotInitialized);
                }
                if avatar_url.is_empty() {
                    return Err(ClientDomainError::InvalidAvatarUrl);
                }

                sink.write(
                    ClientProfileEvent::AvatarUpdated {
                        avatar_url,
                        crop_coordinates,
                    },
                    self,
                )
                .await;
            }
            ClientProfileCommand::AppendStylePreferences { selected_styles } => {
                if !self.is_initialized {
                    return Err(ClientDomainError::ProfileNotInitialized);
                }

                let mut known_styles: HashSet<String> =
                    self.selected_styles.iter().cloned().collect();
                let new_styles: Vec<String> = selected_styles
                    .into_iter()
                    .filter(|style| known_styles.insert(style.clone()))
                    .collect();

                if !new_styles.is_empty() {
                    sink.write(
                        ClientProfileEvent::StylePreferencesUpdated {
                            selected_styles: new_styles,
                            updated_at: Utc::now(),
                        },
                        self,
                    )
                    .await;
                }
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
            ClientProfileEvent::AvatarUpdated { avatar_url, .. } => {
                self.avatar_url = Some(avatar_url);
                // Optionally, you can store crop_coordinates in the aggregate if needed
            }
            ClientProfileEvent::StylePreferencesUpdated {
                selected_styles, ..
            } => {
                self.selected_styles.extend(selected_styles);
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
            ClientProfileEvent::AvatarUpdated { .. } => "AvatarUpdated".to_string(),
            ClientProfileEvent::StylePreferencesUpdated { .. } => {
                "StylePreferencesUpdated".to_string()
            }
            ClientProfileEvent::BarberFollowed { .. } => "BarberFollowed".to_string(),
        }
    }

    fn event_version(&self) -> String {
        match self {
            ClientProfileEvent::ClientProfileInitialized { .. } => "1.0".to_string(),
            ClientProfileEvent::AvatarUpdated { .. } => "1.0".to_string(),
            ClientProfileEvent::StylePreferencesUpdated { .. } => "1.0".to_string(),
            ClientProfileEvent::BarberFollowed { .. } => "1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use cqrs_es::{
        Aggregate, CqrsFramework, EventStore, event_sink::EventSink, mem_store::MemStore,
    };
    use uuid::Uuid;

    use super::{ClientProfileAggregate, ClientProfileCommand};
    use crate::domain::user::events::ClientProfileEvent;

    #[tokio::test]
    async fn style_preference_command_appends_unique_resolved_styles() {
        let client_id = Uuid::new_v4();
        let mut aggregate = ClientProfileAggregate {
            client_id: Some(client_id),
            is_initialized: true,
            selected_styles: vec!["fade".to_string()],
            ..Default::default()
        };
        let sink = EventSink::<ClientProfileAggregate>::default();

        aggregate
            .handle(
                ClientProfileCommand::AppendStylePreferences {
                    selected_styles: vec![
                        "curly".to_string(),
                        "curly".to_string(),
                        "fade".to_string(),
                        "textured".to_string(),
                    ],
                },
                &(),
                &sink,
            )
            .await
            .unwrap();

        let events = sink.collect().await;
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            ClientProfileEvent::StylePreferencesUpdated { selected_styles, .. }
                if selected_styles == &["curly".to_string(), "textured".to_string()]
        ));
        assert_eq!(aggregate.selected_styles, ["fade", "curly", "textured"]);

        let event_json = serde_json::to_string(&events[0]).unwrap();
        assert!(!event_json.contains("gents-curly-top"));
        assert!(!event_json.contains("gents-textured-fade"));
    }

    #[test]
    fn replaying_style_preference_events_restores_the_same_state() {
        let client_id = Uuid::new_v4();
        let mut aggregate = ClientProfileAggregate::default();
        Aggregate::apply(
            &mut aggregate,
            ClientProfileEvent::ClientProfileInitialized {
                client_id,
                created_at: Utc::now(),
            },
        );
        Aggregate::apply(
            &mut aggregate,
            ClientProfileEvent::StylePreferencesUpdated {
                selected_styles: vec!["curly".to_string(), "volume".to_string()],
                updated_at: Utc::now(),
            },
        );

        assert_eq!(
            aggregate.selected_styles,
            ["curly".to_string(), "volume".to_string()]
        );
    }

    #[tokio::test]
    async fn style_preferences_require_an_initialized_profile() {
        let mut aggregate = ClientProfileAggregate::default();
        let sink = EventSink::<ClientProfileAggregate>::default();

        let result = aggregate
            .handle(
                ClientProfileCommand::AppendStylePreferences {
                    selected_styles: vec!["fade".to_string()],
                },
                &(),
                &sink,
            )
            .await;

        assert!(matches!(
            result,
            Err(crate::domain::errors::ClientDomainError::ProfileNotInitialized)
        ));
        assert!(sink.collect().await.is_empty());
    }

    #[tokio::test]
    async fn cqrs_persists_style_deltas_and_replays_appended_state() {
        let client_id = Uuid::new_v4();
        let aggregate_id = client_id.to_string();
        let store = MemStore::<ClientProfileAggregate>::default();
        let cqrs = CqrsFramework::new(store.clone(), Vec::new(), ());

        cqrs.execute(
            &aggregate_id,
            ClientProfileCommand::InitializeProfile { client_id },
        )
        .await
        .unwrap();
        cqrs.execute(
            &aggregate_id,
            ClientProfileCommand::AppendStylePreferences {
                selected_styles: vec!["fade".to_string(), "textured".to_string()],
            },
        )
        .await
        .unwrap();
        cqrs.execute(
            &aggregate_id,
            ClientProfileCommand::AppendStylePreferences {
                selected_styles: vec!["curly".to_string(), "fade".to_string()],
            },
        )
        .await
        .unwrap();
        cqrs.execute(
            &aggregate_id,
            ClientProfileCommand::AppendStylePreferences {
                selected_styles: Vec::new(),
            },
        )
        .await
        .unwrap();

        let events = store.load_events(&aggregate_id).await.unwrap();
        assert_eq!(events.len(), 3);
        let style_events: Vec<_> = events
            .iter()
            .filter_map(|envelope| match &envelope.payload {
                ClientProfileEvent::StylePreferencesUpdated {
                    selected_styles, ..
                } => Some(selected_styles),
                _ => None,
            })
            .collect();
        assert_eq!(style_events.len(), 2);
        assert_eq!(
            style_events[0],
            &vec!["fade".to_string(), "textured".to_string()]
        );
        assert_eq!(style_events[1], &vec!["curly".to_string()]);

        let replayed = store.load_aggregate(&aggregate_id).await.unwrap();
        assert_eq!(
            replayed.aggregate.selected_styles,
            ["fade", "textured", "curly"]
        );
    }
}
