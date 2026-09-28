use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};
use std::sync::Arc;

use crate::{
    cqrs::ClientProfileCqrsFramework,
    domain::user::{
        account_aggregate::UserAccountAggregate, client_profile_aggregate::ClientProfileCommand,
        events::UserAccountEvent,
    },
};

pub struct EmailVerificationSaga {
    client_cqrs: Arc<ClientProfileCqrsFramework>,
}

impl EmailVerificationSaga {
    pub fn new(client_cqrs: Arc<ClientProfileCqrsFramework>) -> Self {
        Self { client_cqrs }
    }
}

#[async_trait]
impl Query<UserAccountAggregate> for EmailVerificationSaga {
    async fn dispatch(&self, _aggregate_id: &str, events: &[EventEnvelope<UserAccountAggregate>]) {
        for event in events {
            if let UserAccountEvent::EmailVerified { user_id, .. } = &event.payload {
                let command = ClientProfileCommand::InitializeProfile {
                    client_id: *user_id,
                };

                if let Err(error) = self
                    .client_cqrs
                    .execute(&user_id.to_string(), command)
                    .await
                {
                    tracing::error!(
                        user_id = %user_id,
                        ?error,
                        "Failed to initialize client profile after email verification"
                    );
                }
            }
        }
    }
}
