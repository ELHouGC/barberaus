// transport/http/auth_cookies.rs
//
// THIS COMMIT IS A PURE REFACTOR: only the error type changes (ApiError -> AppError).
// Cookie attributes are untouched on purpose. One logical change per commit.
//
// TODO(security, SEPARATE COMMIT): the comments below do not match the code.
//   * the comment says "Secure" but there is no `Secure` attribute (cookie can travel over plain HTTP)
//   * the comment says "SameSite=Strict" but the code sends `SameSite=Lax`
//   Decide deliberately, then change both cookie strings (create + clear) together.
use axum::http::{HeaderMap, HeaderValue, header};

use crate::application::error::AppError;

const SEVEN_DAYS_IN_SECONDS: usize = 604_800;

pub fn create_refresh_cookie(refresh_token: &str) -> Result<HeaderMap, AppError> {
    let mut headers = HeaderMap::new();

    // Secure HttpOnly cookie definition
    // SameSite=Strict prevents CSRF
    // Path=/api/v1/auth restricts cookie transmission only to auth endpoints (refresh/logout)
    let cookie_value = format!(
        "refresh_token={}; HttpOnly;  SameSite=Lax; Path=/api/v1/auth/; Max-Age={}",
        refresh_token, SEVEN_DAYS_IN_SECONDS
    );

    headers.insert(
        header::SET_COOKIE,
        // This can only fail if the token contains characters illegal in a header.
        // Our own code generates the token, so that is a bug on OUR side -> internal error (500).
        // `AppError::internal` keeps the original error for the logs (the old `|_|` dropped it).
        HeaderValue::from_str(&cookie_value).map_err(AppError::internal)?,
    );

    Ok(headers)
}

pub fn clear_refresh_cookie() -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Max-Age=0 instantly deletes the cookie in the user's browser.
    // Attributes (esp. Path) must match the creation cookie or the browser keeps the old one.
    let cookie_value = "refresh_token=; HttpOnly;  SameSite=Lax; Path=/api/v1/auth/; Max-Age=0";

    headers.insert(header::SET_COOKIE, HeaderValue::from_static(cookie_value));

    headers
}