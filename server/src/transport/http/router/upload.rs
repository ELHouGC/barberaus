use axum::{Router, routing::get};
use std::sync::Arc;

use crate::{
    application::services::cloudinary_service::CloudinaryService,
    transport::http::handlers::cloudinary_upload::handle_get_upload_signature,
};

pub fn create_router(cloudinary: Arc<CloudinaryService>) -> Router {
    Router::new()
        .route("/api/uploads/signature", get(handle_get_upload_signature))
        .with_state(cloudinary)
}

#[cfg(test)]
mod tests {
    use super::create_router;
    use crate::application::services::cloudinary_service::CloudinaryService;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_upload_signature_returns_cloudinary_signature_data() {
        let api_secret = "test-api-secret";
        let router = create_router(Arc::new(CloudinaryService::from_credentials(
            "test-cloud".to_string(),
            "test-api-key".to_string(),
            api_secret.to_string(),
        )));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/uploads/signature")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "application/json"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let signature_data: Value = serde_json::from_slice(&body).unwrap();
        let timestamp = signature_data["timestamp"].as_u64().unwrap();
        let expected_string = format!("folder=avatars&timestamp={timestamp}{api_secret}");
        let expected_signature = format!("{:x}", Sha256::digest(expected_string.as_bytes()));

        assert_eq!(signature_data["cloud_name"], "test-cloud");
        assert_eq!(signature_data["api_key"], "test-api-key");
        assert_eq!(signature_data["folder"], "avatars");
        assert_eq!(signature_data["signature"], expected_signature);
    }
}
