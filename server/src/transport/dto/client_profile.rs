use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use crate::error::ApiError;

const MAX_AVATAR_URL_LEN: usize = 2048;
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "avif"];

#[derive(Debug, Deserialize)]
pub struct UpdateAvatarRequest {
    pub user_id: Uuid,
    pub avatar_url: String,
}

/// Clean application-layer structure containing only what the business logic needs.
#[derive(Debug, Clone)]
pub struct AvatarUpload {
    pub user_id: Uuid,
    pub avatar_url: String,
}

impl UpdateAvatarRequest {
    /// Transport-level validation: syntax, HTTPS enforcement, Cloudinary domain, 
    /// cloud name match, delivery path structure, and allowed image extensions.
    pub fn validate(self, expected_cloud_name: &str) -> Result<AvatarUpload, ApiError> {
        let raw = self.avatar_url.trim();
        let mut errors = Vec::new();

        if raw.is_empty() {
            errors.push(("avatar_url", "validation.required"));
        } else if raw.len() > MAX_AVATAR_URL_LEN {
            errors.push(("avatar_url", "validation.too_long"));
        } else {
            match Url::parse(raw) {
                Ok(u) => {
                    // 1. Enforce HTTPS scheme
                    if u.scheme() != "https" {
                        errors.push(("avatar_url", "validation.url_must_be_https"));
                    }

                    // 2. Validate that the host belongs to Cloudinary CDN (and block backend API subdomains)
                    match u.host_str() {
                        Some(host) if host.ends_with("cloudinary.com") => {
                            if host.starts_with("api.") {
                                errors.push(("avatar_url", "validation.invalid_image_domain"));
                            }
                        }
                        Some(_) => {
                            errors.push(("avatar_url", "validation.invalid_image_domain"));
                        }
                        None => {
                            errors.push(("avatar_url", "validation.url_invalid"));
                        }
                    }

                    // 3. Extract and validate path segments
                    let segments: Vec<&str> = u
                        .path_segments()
                        .map(|s| s.collect())
                        .unwrap_or_default();

                    if segments.len() < 4 {
                        errors.push(("avatar_url", "validation.invalid_cloudinary_structure"));
                    } else {
                        let cloud_name = segments[0];
                        let asset_type = segments[1];
                        let delivery_type = segments[2];

                        // Enforce that the asset belongs to YOUR specific Cloudinary cloud
                        if cloud_name != expected_cloud_name {
                            errors.push(("avatar_url", "validation.invalid_cloud_name"));
                        }
                        if asset_type != "image" {
                            errors.push(("avatar_url", "validation.invalid_asset_type"));
                        }
                        if delivery_type != "upload" {
                            errors.push(("avatar_url", "validation.invalid_delivery_type"));
                        }
                    }

                    // 4. Validate File Extension
                    let path = u.path();
                    let extension = path
                        .split('.')
                        .last()
                        .unwrap_or("")
                        .to_lowercase();

                    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
                        errors.push(("avatar_url", "validation.invalid_file_extension"));
                    }
                }
                Err(_) => {
                    errors.push(("avatar_url", "validation.url_invalid"));
                }
            }
        }

        if errors.is_empty() {
            Ok(AvatarUpload {
                user_id: self.user_id,
                avatar_url: self.avatar_url.trim().to_string(),
            })
        } else {
            Err(ApiError::InvalidInput(format!(
                "Validation errors: {:?}",
                errors
                    .into_iter()
                    .map(|(field, msg)| format!("{}: {}", field, msg))
                    .collect::<Vec<_>>()
            )))
        }
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