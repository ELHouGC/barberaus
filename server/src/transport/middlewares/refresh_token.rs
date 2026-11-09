// transport/http/extractors/refresh_token.rs
//
// STYLE RULES FOR THIS FILE
// -------------------------
// 1. NEVER print or log a token, a cookie header or the whole request.
//    A refresh token in a log file is a stolen session waiting to happen.
//    (The old `println!`s dumped `parts` -- including the Cookie and
//    Authorization headers -- and the raw token.)
// 2. One tiny function per source (header, cookie). Each returns
//    `Option<&str>`: "a non-empty token, or nothing". The extractor then just
//    chains them: header first, cookie as a fallback.

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use crate::application::error::{AppError, AuthError};

#[derive(Clone)]
pub struct RefreshToken(pub String);

// We do NOT `#[derive(Debug)]` on purpose: the derived version prints the token.
// If someone later writes `#[instrument]` on a handler taking a `RefreshToken`
// (or logs it with `{:?}`), the secret would leak. This manual version cannot.
impl std::fmt::Debug for RefreshToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RefreshToken(<redacted>)")
    }
}

#[allow(dead_code)]
impl RefreshToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Source 1: the custom `X-Refresh-Token` header (non-browser clients).
///
/// `Option<&str>` borrows from `parts`: no allocation until we really need an owned String.
fn from_header(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get("x-refresh-token")? // header absent -> return None
        .to_str()
        .ok() // header not valid text -> None
        .filter(|token| !token.is_empty()) // empty value -> None
}

/// Source 2: the `refresh_token=<token>` cookie (browser clients).
fn from_cookie(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get_all(header::COOKIE) // ALL Cookie headers: HTTP/2 may split them
        .iter()
        .filter_map(|value| value.to_str().ok()) // skip non-text headers
        .flat_map(|value| value.split(';')) // "a=1; refresh_token=x" -> ["a=1", " refresh_token=x"]
        .filter_map(|cookie| cookie.trim().strip_prefix("refresh_token="))
        .find(|token| !token.is_empty()) // first non-empty match
}

impl<S> FromRequestParts<S> for RefreshToken
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Re-borrow as a shared reference: the helpers only read.
        let parts: &Parts = parts;

        from_header(parts)
            .or_else(|| from_cookie(parts)) // only evaluated if the header gave nothing
            .map(|token| RefreshToken(token.to_owned()))
            .ok_or(AppError::Auth(AuthError::MissingToken))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    /// Builds a request with the given headers and runs the extractor on it.
    async fn extract(headers: &[(&str, &str)]) -> Result<RefreshToken, AppError> {
        let mut builder = Request::builder();
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let (mut parts, _body) = builder.body(()).unwrap().into_parts();
        RefreshToken::from_request_parts(&mut parts, &()).await
    }

    #[tokio::test]
    async fn reads_the_custom_header() {
        let token = extract(&[("x-refresh-token", "abc")]).await.unwrap();
        assert_eq!(token.as_str(), "abc");
    }

    #[tokio::test]
    async fn reads_the_cookie_among_other_cookies() {
        let token = extract(&[("cookie", "theme=dark; refresh_token=xyz; lang=fr")])
            .await
            .unwrap();
        assert_eq!(token.as_str(), "xyz");
    }

    #[tokio::test]
    async fn header_wins_over_cookie() {
        let token = extract(&[
            ("x-refresh-token", "from-header"),
            ("cookie", "refresh_token=from-cookie"),
        ])
        .await
        .unwrap();
        assert_eq!(token.as_str(), "from-header");
    }

    #[tokio::test]
    async fn empty_values_count_as_missing() {
        let err = extract(&[("x-refresh-token", ""), ("cookie", "refresh_token=")])
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Auth(AuthError::MissingToken)));
    }

    #[test]
    fn debug_never_prints_the_token() {
        let token = RefreshToken("super-secret".to_string());
        assert!(!format!("{token:?}").contains("super-secret"));
    }
}
