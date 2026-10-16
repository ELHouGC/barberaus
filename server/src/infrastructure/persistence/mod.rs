use crate::application::ports::repo_error::RepoError;

pub mod pg_auth_repository;
pub mod pg_email_verification_repository;

/// Converts a `sqlx::Error` into our `RepoError`.
///
/// Usage: `.map_err(db_err("email_verifications.create"))?`
///
/// It returns a *closure* (`impl FnOnce(sqlx::Error) -> RepoError`) because
/// `map_err` expects a function that receives the error. We pre-fill the
/// `operation` name and `move` it into the closure.
fn db_err(operation: &'static str) -> impl FnOnce(sqlx::Error) -> RepoError {
    move |err| {
        // Errors that mean "the database cannot be reached", as opposed to
        // "the query itself is wrong". We must test BEFORE moving `err` into the Box.
        let unavailable = matches!(
            err,
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_)
        );
        let source = Box::new(err);

        if unavailable {
            RepoError::Unavailable { operation, source }
        } else {
            RepoError::Unexpected { operation, source }
        }
    }
}