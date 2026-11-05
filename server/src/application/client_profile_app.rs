// app/client_profile.rs
use cqrs_es::persist::ViewRepository;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{
        error::{AppError, Resource},
        services::{
            cloudinary_service::CloudinaryService, style_photo_catalog::StylePhotoCatalogService,
        },
    },
    cqrs::{ClientProfileCqrsFramework, ClientProfileViewRepository},
    domain::{
        errors::ClientDomainError,
        user::{client_profile_aggregate::ClientProfileCommand, value_objects::AvatarUpload},
    },
    transport::dto::client_profile::{SignatureResponse, StylePreferencesResponse},
};

pub struct ClientProfileApp {
    cqrs: Arc<ClientProfileCqrsFramework>,
    view_repo: Arc<ClientProfileViewRepository>,
    pub cloudinary: Arc<CloudinaryService>,
}

impl ClientProfileApp {
    pub fn new(
        cqrs: Arc<ClientProfileCqrsFramework>,
        cloudinary: Arc<CloudinaryService>,
        view_repo: Arc<ClientProfileViewRepository>,
    ) -> Self {
        Self {
            cqrs,
            view_repo,
            cloudinary,
        }
    }

    pub async fn generate_avatar_signature(
        &self,
        user_id: Uuid,
    ) -> Result<SignatureResponse, AppError> {
        let folder = crate::transport::dto::avatar_url::avatar_folder(user_id);
        self.cloudinary.generate_upload_signature(&folder)
    }

    pub async fn update_avatar(&self, input: AvatarUpload) -> Result<(), AppError> {
        let aggregate_id = input.user_id.to_string();

        let command = ClientProfileCommand::UpdateAvatar {
            user_id: aggregate_id.clone(),
            avatar_url: input.avatar_url,
            crop_coordinates: input.crop_coordinates,
        };

        self.cqrs.execute(&aggregate_id, command).await?;

        Ok(())
    }

    pub async fn append_style_preferences(
        &self,
        user_id: Uuid,
        selected_photo_ids: Vec<String>,
    ) -> Result<StylePreferencesResponse, AppError> {
        let selected_styles = StylePhotoCatalogService::resolve_style_ids(&selected_photo_ids)?;
        let aggregate_id = user_id.to_string();
        
        self.cqrs
            .execute(
                &aggregate_id,
                ClientProfileCommand::AppendStylePreferences { selected_styles },
            )
            .await?;

        let view = self
            .view_repo
            .load(&aggregate_id)
            .await
            .map_err(AppError::internal)?
            .ok_or(AppError::NotFound(Resource::User))?;
        let client_profile = view.client_profile.ok_or(AppError::ClientProfile(
            ClientDomainError::ProfileNotInitialized,
        ))?;

        Ok(StylePreferencesResponse {
            selected_styles: client_profile.selected_styles,
        })
    }
}
