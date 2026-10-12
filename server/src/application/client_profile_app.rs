// app/client_profile.rs
use std::sync::Arc;
use crate::{
    application::services::cloudinary_service::CloudinaryService, cqrs::ClientProfileCqrsFramework, domain::user::client_profile_aggregate::ClientProfileCommand, error::ApiError, transport::dto::client_profile::AvatarUpload};

pub struct ClientProfileApp {
    cqrs: Arc<ClientProfileCqrsFramework>, // your CqrsFramework alias
    pub cloudinary: Arc<CloudinaryService>,
}

impl ClientProfileApp {
    pub fn new(cqrs: Arc<ClientProfileCqrsFramework>, cloudinary: Arc<CloudinaryService>) -> Self {
        Self { cqrs, cloudinary }
    }




    pub async fn update_avatar(&self, input: AvatarUpload) -> Result<(), ApiError> {
       
        let aggregate_id = input.user_id.to_string();
        let command = ClientProfileCommand::UpdateAvatar {
            user_id: aggregate_id.clone(),
            avatar_url: input.avatar_url.into(),
        };

        self.cqrs
            .execute(&aggregate_id, command)
            .await?;

        Ok(())
    }

}