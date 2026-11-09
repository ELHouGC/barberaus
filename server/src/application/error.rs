// application/error.rs
use uuid::Uuid;
use crate::domain::errors::{BarberDomainError, ClientDomainError, UserDomainError};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("authentication: {0:?}")]   Auth(AuthError),
    #[error("access: {0:?}")]           Access(AccessError),
    #[error("verification: {0:?}")]     Verification(VerificationError),
    #[error("validation failed")]       Validation(Vec<Violation>),
    #[error(transparent)]               ClientProfile(ClientDomainError),
    #[error(transparent)]               BarberProfile(BarberDomainError),
    #[error(transparent)]               User(UserDomainError),
    #[error("not found: {0:?}")]        NotFound(Resource),
    #[error("conflict: {0:?}")]         Conflict(ConflictKind),
    #[error("concurrent modification")] ConcurrentModification,
    #[error("{dependency} unavailable")] Unavailable { dependency: &'static str, #[source] source: BoxError },
    #[error("internal error")]          Internal(#[source] BoxError),
}

#[derive(Debug)] pub enum AuthError {
    MissingToken, InvalidToken, TokenExpired, InvalidCredentials,
    AccountNotVerified { user_id: Uuid },
}
#[derive(Debug)] pub enum  AccessError { Forbidden, TenantContextRequired, MembershipNotFound }
#[derive(Debug)] pub enum VerificationError { InvalidToken, Expired }
#[derive(Debug)] pub enum Resource { User , BarberProfile }
#[derive(Debug)] pub enum ConflictKind { EmailExists, AccountAlreadyVerified  }

/// One failing field: JSON pointer + i18n key.
#[derive(Debug, serde::Serialize)]
pub struct Violation { pub pointer: String, pub key: &'static str }

impl AppError {
    pub fn internal(e: impl Into<BoxError>) -> Self { Self::Internal(e.into()) }
    pub fn unavailable(dependency: &'static str, e: impl Into<BoxError>) -> Self {
       Self::Unavailable { dependency, source: e.into() }
   }
}

impl From<ClientDomainError> for AppError {
    fn from(e: ClientDomainError) -> Self{
        Self::ClientProfile(e)
    }
}
impl From<UserDomainError>   for AppError { 
    fn from(e: UserDomainError)   -> Self { 
        Self::User(e) 
    } 
}

impl<E> From<cqrs_es::AggregateError<E>> for AppError
where E: std::error::Error + Into<AppError> {
    fn from(e: cqrs_es::AggregateError<E>) -> Self {
        use cqrs_es::AggregateError as A;
        match e {
            A::UserError(domain)            => domain.into(),
            A::AggregateConflict            => AppError::ConcurrentModification,
            A::DatabaseConnectionError(src) => AppError::Unavailable { dependency: "event_store", source: src },
            A::DeserializationError(src)    => AppError::Internal(src),
            A::UnexpectedError(src)         => AppError::Internal(src),
        }
    }
}

use crate::application::ports::repo_error::RepoError;

impl From<RepoError> for AppError {
    fn from(e: RepoError) -> Self {
        // Patterns with `{ .. }` don't move `e`, so we can still box it in the arm.
        // We box the whole RepoError (not just its source) to keep `operation` in the logs.
        match e {
            RepoError::Unavailable { .. } => AppError::Unavailable {
                dependency: "database",
                source: Box::new(e),
            },
            RepoError::Unexpected { .. } => AppError::Internal(Box::new(e)),
        }
    }
}