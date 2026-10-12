use crate::domain::errors::ClientDomainError;
use crate::error::api_error::ApiError;
use cqrs_es::AggregateError;

impl From<AggregateError<ClientDomainError>> for ApiError {
    fn from(error: AggregateError<ClientDomainError>) -> Self {
        match error {
            // 1. Unwrap the domain business logic errors
            AggregateError::UserError(domain_err) => match domain_err {
                ClientDomainError::AlreadyInitialized => {
                    ApiError::InvalidInput("Client profile is already initialized".to_string())
                }
                ClientDomainError::ProfileNotInitialized => ApiError::InvalidInput(
                    "Client profile must be initialized before onboarding".to_string(),
                ),
                ClientDomainError::AlreadyOnboarded => ApiError::InvalidInput(
                    "Client onboarding has already been completed".to_string(),
                ),
                ClientDomainError::CannotFollowSelf => {
                    ApiError::InvalidInput("Cannot follow your own profile".to_string())
                }
                ClientDomainError::AlreadyFollowing => {
                    ApiError::InvalidInput("Already following this barber".to_string())
                }
                ClientDomainError::InvalidState => ApiError::InternalServerError,
            },

            // 2. Concurrency conflict / Optimistic lock failure
            AggregateError::AggregateConflict => ApiError::InvalidInput(
                "A concurrent update occurred. Please try again.".to_string(),
            ),

            // 4. Any other unexpected CQRS framework errors
            _ => ApiError::InternalServerError,
        }
    }
}
