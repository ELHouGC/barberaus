use axum::{Json, extract::State};

use crate::{error::ApiError, state::AppState, transport::{dto::auth::{CompleteOnboardingRequest, OnboardingResponse}, middlewares::auth_context::AuthContext, response::ApiResponse}};






pub async fn complete_onboarding(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CompleteOnboardingRequest>,
) -> Result<Json<ApiResponse<OnboardingResponse>>, ApiError> {
    // 1. Pre-repository / Pre-aggregate validation check
    payload.validate(auth.user_id())?;

    // 2. Convert validated DTO into aggregate command
    let command = payload.into_command();

    // 3. Execute command on ClientProfile CQRS framework & map DomainError via domain_mapper
    state
        .client_profile_cqrs_framework
        .execute(&auth.user_id().to_string(), command)
        .await
        .map_err(|err| ApiError::from(err))?;

    // 4. Return standard ApiResponse envelope
    let response = OnboardingResponse {
        client_id: auth.user_id(),
        message: "Client onboarding completed successfully".to_string(),
    };

    Ok(Json(ApiResponse::success(response)))
}