use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorPayload>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        user_id: Option<Uuid>,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiErrorPayload {
                code: code.into(),
                message: message.into(),
                user_id,
            }),
        }
    }
}
