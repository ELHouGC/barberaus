// application/ports/repo_error.rs
//
// WHY THIS FILE EXISTS
// --------------------
// A repository can only fail for *technical* reasons: the database is down,
// the pool is exhausted, a query is malformed...
// It must NOT know about HTTP, status codes or business rules ("token expired").
// So repositories return `RepoError`, and the *service* layer decides what
// each outcome means for the business (via `AppError`).
//
// WHY IT LIVES IN `application/ports/` AND NOT IN `infrastructure/`
// -----------------------------------------------------------------
// The trait `EmailVerificationRepo` (the "port") lives in `application/` and
// mentions `RepoError` in its signatures. Dependencies must point inward
// (infrastructure -> application), never the other way round, so the error
// type has to live next to the port.
//
// That is also why `source` is a `Box<dyn Error>` and NOT a `sqlx::Error`:
// this file must not depend on sqlx. The sqlx-specific code that builds a
// `RepoError` lives in the Postgres repository.

/// A boxed error of any type. `Send + Sync` is required so the error can
/// cross thread boundaries (Tokio moves futures between threads).
type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// `thiserror` generates the `Display` and `Error` impls from the
/// `#[error("...")]` attributes, so we don't write them by hand.
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    /// The storage cannot be reached right now (pool timeout, connection lost).
    /// Likely temporary -> mapped to HTTP 503 (see `From<RepoError> for AppError`).
    #[error("storage unavailable during `{operation}`")]
    Unavailable {
        /// Which repository operation failed, e.g. "email_verifications.create".
        /// This is our "stack trace": it tells us *where* it failed when the
        /// error is logged later, far away from the repository.
        operation: &'static str,
        /// The original error (kept for logs, never sent to the client).
        #[source]
        source: BoxError,
    },

    /// Anything else (bad SQL, constraint violation, decoding error...).
    /// Almost always a bug or a data problem -> mapped to HTTP 500.
    #[error("unexpected storage failure during `{operation}`")]
    Unexpected {
        operation: &'static str,
        #[source]
        source: BoxError,
    },
}
