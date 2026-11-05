use axum::{Router, routing::get};

use crate::transport::http::handlers::style_photos::get_style_photos;

pub fn create_router() -> Router {
    Router::new().route("/api/v1/style-photos", get(get_style_photos))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    use super::create_router;

    #[tokio::test]
    async fn catalogue_is_public_and_uses_the_success_envelope() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/style-photos?category=gents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"].as_array().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn missing_category_uses_the_standard_problem_validation_response() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/style-photos")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.headers()["content-type"],
            "application/problem+json"
        );
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "INVALID_INPUT");
        assert_eq!(json["errors"][0]["pointer"], "/category");
        assert_eq!(json["errors"][0]["key"], "validation.required");
    }

    #[tokio::test]
    async fn unsupported_category_uses_the_standard_problem_validation_response() {
        let response = create_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/style-photos?category=children")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "INVALID_INPUT");
        assert_eq!(json["errors"][0]["pointer"], "/category");
        assert_eq!(json["errors"][0]["key"], "validation.unsupported_category");
    }
}
