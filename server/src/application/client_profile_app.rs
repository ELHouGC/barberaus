// app/client_profile.rs
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{error::AppError, services::cloudinary_service::CloudinaryService}, cqrs::ClientProfileCqrsFramework, domain::user::client_profile_aggregate::ClientProfileCommand, transport::dto::client_profile::{AvatarUpload, SignatureResponse}};

pub struct ClientProfileApp {
    cqrs: Arc<ClientProfileCqrsFramework>, // your CqrsFramework alias
    pub cloudinary: Arc<CloudinaryService>,
}

impl ClientProfileApp {
    pub fn new(cqrs: Arc<ClientProfileCqrsFramework>, cloudinary: Arc<CloudinaryService>) -> Self {
        Self { cqrs, cloudinary }
    }

pub async fn generate_avatar_signature(&self, user_id: Uuid) -> Result<SignatureResponse, AppError> {
        let folder = crate::transport::dto::avatar_url::avatar_folder(user_id);
        self.cloudinary.generate_upload_signature(&folder)
}

pub async fn update_avatar(&self, input: AvatarUpload) -> Result<(), AppError> {
    let aggregate_id = input.user_id.to_string();
    let command = ClientProfileCommand::UpdateAvatar {
        user_id: aggregate_id.clone(),
        avatar_url: input.avatar_url.into_string(),
    };

    self.cqrs
        .execute(&aggregate_id, command)
        .await?;

    Ok(())
}

}