use crate::{
    application::error::{AppError, Violation},
    domain::user::value_objects::{AvatarUpload, CropCoordinates},
    transport::dto::avatar_url::{AvatarPolicy, AvatarUrl, AvatarUrlError},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdateAvatarRequest {
    pub avatar_url: String,
    pub crop_coordinates: CropCoordinates,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStylePreferencesRequest {
    pub selected_style_photo_ids: Option<Vec<String>>,
}

impl UpdateStylePreferencesRequest {
    pub fn selected_photo_ids(self) -> Result<Vec<String>, AppError> {
        self.selected_style_photo_ids.ok_or_else(|| {
            AppError::Validation(vec![Violation {
                pointer: "/selected_style_photo_ids".to_string(),
                key: "validation.required",
            }])
        })
    }
}

#[derive(Debug, Serialize)]
pub struct StylePreferencesResponse {
    pub selected_styles: Vec<String>,
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

#[cfg(test)]
mod style_preference_tests {
    use super::{StylePreferencesResponse, UpdateStylePreferencesRequest};
    use crate::application::error::AppError;
    use crate::transport::response::ApiResponse;

    #[test]
    fn missing_photo_ids_use_the_standard_validation_error() {
        let request: UpdateStylePreferencesRequest = serde_json::from_str("{}").unwrap();
        let Err(AppError::Validation(violations)) = request.selected_photo_ids() else {
            panic!("expected a validation error");
        };

        assert_eq!(violations[0].pointer, "/selected_style_photo_ids");
        assert_eq!(violations[0].key, "validation.required");
    }

    #[test]
    fn empty_photo_selection_is_valid() {
        let request: UpdateStylePreferencesRequest =
            serde_json::from_str(r#"{"selected_style_photo_ids":[]}"#).unwrap();
        let selected = request.selected_photo_ids().unwrap();

        assert!(selected.is_empty());
    }

    #[test]
    fn response_uses_the_success_envelope_and_resolved_styles() {
        let response = ApiResponse::success(StylePreferencesResponse {
            selected_styles: vec!["curly".to_string(), "volume".to_string()],
        });
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["success"], true);
        assert_eq!(
            value["data"]["selected_styles"],
            serde_json::json!(["curly", "volume"])
        );
        assert!(value["data"].get("selected_style_photo_ids").is_none());
    }
}
