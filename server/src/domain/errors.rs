use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum UserDomainError {
    // -------------------------------------------------------------------------
    // UserAccount Errors
    // -------------------------------------------------------------------------
    #[error("User account already exists")]
    AlreadyExists,

    #[error("User email cannot be empty or invalid")]
    InvalidEmail,

    #[error("User account not found")]
    UserNotFound,

    #[error("User email is already verified")]
    AlreadyVerified,

    #[error("User email must be verified before performing this action")]
    UserNotVerified,

    #[error("Requested profile already exists on this account")]
    ProfileAlreadyExists,

    // -------------------------------------------------------------------------
    // BarberProfile Errors
    // -------------------------------------------------------------------------
    #[error("Target profile is not registered as a barber")]
    NotABarber,

    // -------------------------------------------------------------------------
    // Infrastructure & Generic Domain Invariants
    // -------------------------------------------------------------------------
    #[error("Aggregate reached an invalid internal state")]
    InvalidState,
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum ClientDomainError {
    #[error("Client profile is already initialized")]
    AlreadyInitialized,

    #[error("Client profile must be initialized before onboarding")]
    ProfileNotInitialized,

    #[error("Client onboarding has already been completed")]
    AlreadyOnboarded,

    #[error("Cannot follow your own profile")]
    CannotFollowSelf,

    #[error("Already following this barber")]
    AlreadyFollowing,

    #[error("Aggregate reached an invalid internal state")]
    InvalidState,
    #[error("Invalid avatar URL provided")]
    InvalidAvatarUrl,
}


#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum BarberDomainError {
    #[error("Barber profile is already initialized")]
    AlreadyInitialized,

    #[error("Barber profile must be initialized before onboarding")]
    ProfileNotInitialized,

    #[error("Barber onboarding has already been completed")]
    AlreadyOnboarded,

    #[error("Cannot follow your own profile")]
    CannotFollowSelf,

    #[error("Already following this barber")]
    AlreadyFollowing,

    #[error("Aggregate reached an invalid internal state")]
    InvalidState,
    #[error("Invalid avatar URL provided")]
    InvalidAvatarUrl,
}
