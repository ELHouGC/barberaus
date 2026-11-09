use crate::application::error::AppError;
use crate::transport::dto::client_profile::SignatureResponse;
use sha2::{Digest, Sha256};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH}; // or wherever SignatureResponse lives

pub struct CloudinaryService {
    pub cloud_name: String,
    pub api_key: String,
    pub api_secret: String,
}

impl CloudinaryService {
    pub fn new() -> Result<Self, AppError> {
        let cloud_name = env::var("CLOUDINARY_CLOUD_NAME").map_err(|e| {
            AppError::internal(format!("Missing CLOUDINARY_CLOUD_NAME env var: {e}"))
        })?;

        let api_key = env::var("CLOUDINARY_API_KEY")
            .map_err(|e| AppError::internal(format!("Missing CLOUDINARY_API_KEY env var: {e}")))?;

        let api_secret = env::var("CLOUDINARY_API_SECRET").map_err(|e| {
            AppError::internal(format!("Missing CLOUDINARY_API_SECRET env var: {e}"))
        })?;

        Ok(Self::from_credentials(cloud_name, api_key, api_secret))
    }

    pub fn from_credentials(cloud_name: String, api_key: String, api_secret: String) -> Self {
        Self {
            cloud_name,
            api_key,
            api_secret,
        }
    }

    pub fn generate_upload_signature(&self, folder: &str) -> Result<SignatureResponse, AppError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::internal(e))?
            .as_secs();

        // 1. Sort parameters alphabetically: folder, then timestamp
        // Format string to sign: folder={folder}&timestamp={timestamp}{api_secret}
        let string_to_sign = format!(
            "folder={}&timestamp={}{}",
            folder, timestamp, self.api_secret
        );

        // 2. Generate SHA-256 hash hex digest
        let mut hasher = Sha256::new();
        hasher.update(string_to_sign.as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        Ok(SignatureResponse {
            signature,
            timestamp,
            api_key: self.api_key.clone(),
            cloud_name: self.cloud_name.clone(),
            folder: folder.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CloudinaryService;
    use sha2::{Digest, Sha256};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn generate_upload_signature_returns_expected_signature_and_metadata() {
        let api_secret = "test-api-secret";
        let service = CloudinaryService::from_credentials(
            "test-cloud".to_string(),
            "test-api-key".to_string(),
            api_secret.to_string(),
        );

        let response = service
            .generate_upload_signature("test-folder")
            .expect("signature generation should succeed");

        let string_to_sign = format!(
            "folder=test-folder&timestamp={}{}",
            response.timestamp, api_secret
        );
        let expected_signature = format!("{:x}", Sha256::digest(string_to_sign.as_bytes()));
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_secs();

        assert_eq!(response.signature, expected_signature);
        assert_eq!(response.signature.len(), 64);
        assert_eq!(response.api_key, "test-api-key");
        assert_eq!(response.cloud_name, "test-cloud");
        assert_eq!(response.folder, "test-folder");
        assert!(response.timestamp <= current_timestamp);
        assert!(current_timestamp - response.timestamp <= 1);
    }
}
