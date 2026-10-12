use axum::http::{HeaderMap, HeaderValue, header};

const SEVEN_DAYS_IN_SECONDS: usize = 604_800;

pub fn create_refresh_cookie(refresh_token: &str) -> Result<HeaderMap, crate::error::ApiError> {
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
        HeaderValue::from_str(&cookie_value).map_err(|_| crate::error::ApiError::Unexpected)?,
    );

    Ok(headers)
}

pub fn clear_refresh_cookie() -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Max-Age=0 instantly deletes the cookie in the user's browser
    let cookie_value = "refresh_token=; HttpOnly;  SameSite=Lax; Path=/api/v1/auth/; Max-Age=0";

    headers.insert(header::SET_COOKIE, HeaderValue::from_static(cookie_value));

    headers
}
