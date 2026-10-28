use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{application::error::{AppError, Violation}, domain::user::value_objects::{AvatarUpload, CropCoordinates}, transport::dto::avatar_url::{AvatarPolicy, AvatarUrl, AvatarUrlError}};

#[derive(Debug, Deserialize)]
pub struct UpdateAvatarRequest {
    pub avatar_url: String,
    pub crop_coordinates: CropCoordinates,
}


impl UpdateAvatarRequest {
    pub fn validate(self, expected_cloud_name: &str, user: Uuid) -> Result<AvatarUpload, AppError> {
        let policy = AvatarPolicy {
            cloud_name: expected_cloud_name,
            owner: user,
        };

        let avatar_url = AvatarUrl::parse(&self.avatar_url, &policy).map_err(|err| {
            let key = match err {
                AvatarUrlError::Empty => "validation.required",
                AvatarUrlError::TooLong => "validation.too_long",
                AvatarUrlError::Malformed => "validation.url_invalid",
                AvatarUrlError::NotHttps => "validation.url_must_be_https",
                AvatarUrlError::WrongHost => "validation.invalid_image_domain",
                AvatarUrlError::WrongCloud => "validation.invalid_cloud_name",
                AvatarUrlError::WrongAssetType => "validation.invalid_asset_type",
                AvatarUrlError::WrongDeliveryType => "validation.invalid_delivery_type",
                AvatarUrlError::WrongStructure => "validation.invalid_cloudinary_structure",
                AvatarUrlError::NotOwner => "validation.invalid_owner",
                AvatarUrlError::BadExtension => "validation.invalid_file_extension",
            };

            AppError::Validation(vec![Violation {
                pointer: "/avatar_url".to_string(),
                key,
            }])

        })?;

     
        
        if self.crop_coordinates.width <= 0 || self.crop_coordinates.height <= 0 {
            return Err(AppError::Validation(vec![Violation {
                pointer: "/crop_coordinates".to_string(),
                key: "validation.invalid_crop_dimensions",
            }]));
        }

        Ok(AvatarUpload {
            user_id: user,
            avatar_url: avatar_url.into_string(),
            crop_coordinates: self.crop_coordinates,
        })
    }
}

#[derive(Serialize)]
pub struct SignatureResponse {
    pub signature: String,
    pub timestamp: u64,
    pub api_key: String,
    pub cloud_name: String,
    pub folder: String,
}