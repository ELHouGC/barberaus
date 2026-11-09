//! Avatar URL value object  (suggested location: `domain/avatar_url.rs`).
//!
//! An `AvatarUrl` can only be obtained through `AvatarUrl::parse`, so any function
//! that receives one knows the URL is: https, on our Cloudinary delivery host, in our
//! cloud, an image upload, inside the owner's avatar folder, with an allowed extension.
//!
//! The stored value is the *parsed and normalized* URL, never the raw input:
//! what we validate is exactly what we persist.
//!
//! Cloudinary's delivery-URL layout is encoded here and only here. If the media
//! provider changes, this is the file that changes.

use url::Url;
use uuid::Uuid;

const MAX_LEN: usize = 2048;
const DELIVERY_HOST: &str = "res.cloudinary.com";
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "avif"];

/// Root folder for avatars. The upload signature and the parser share it so they cannot drift.
pub const AVATAR_FOLDER: &str = "avatars";

/// Folder the upload signature must authorise for this user: `avatars/{owner}`.
pub fn avatar_folder(owner: Uuid) -> String {
    format!("{AVATAR_FOLDER}/{owner}")
}

/// Runtime facts the parser needs. Passed in so the domain stays free of configuration.
#[derive(Debug, Clone, Copy)]
pub struct AvatarPolicy<'a> {
    pub cloud_name: &'a str,
    pub owner: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AvatarUrlError {
    #[error("avatar url is empty")]
    Empty,
    #[error("avatar url is too long")]
    TooLong,
    #[error("avatar url is malformed")]
    Malformed,
    #[error("avatar url must use https")]
    NotHttps,
    #[error("avatar url host is not the media delivery host")]
    WrongHost,
    #[error("avatar url belongs to another cloud")]
    WrongCloud,
    #[error("avatar url is not an image asset")]
    WrongAssetType,
    #[error("avatar url is not an uploaded asset")]
    WrongDeliveryType,
    #[error("avatar url path has an unexpected structure")]
    WrongStructure,
    #[error("avatar url is outside the owner's folder")]
    NotOwner,
    #[error("avatar url has a disallowed file extension")]
    BadExtension,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarUrl(String);

impl AvatarUrl {
    /// Returns the first violation found; the URL is machine-generated, so
    /// accumulating every problem would only add noise.
    pub fn parse(raw: &str, policy: &AvatarPolicy<'_>) -> Result<Self, AvatarUrlError> {
        use AvatarUrlError::*;

        let raw = raw.trim();
        if raw.is_empty() {
            return Err(Empty);
        }
        if raw.len() > MAX_LEN {
            return Err(TooLong);
        }

        let url = Url::parse(raw).map_err(|_| Malformed)?;

        if url.scheme() != "https" {
            return Err(NotHttps);
        }
        // Exact match: `ends_with("cloudinary.com")` would accept `evilcloudinary.com`.
        if url.host_str() != Some(DELIVERY_HOST) {
            return Err(WrongHost);
        }
        // Canonical form only: no credentials, custom port, query or fragment.
        if !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(Malformed);
        }

        // [cloud, "image", "upload", ("v123")?, "avatars", "<owner>", "<file>"]
        let segments: Vec<&str> = url.path_segments().ok_or(WrongStructure)?.collect();
        let [cloud, asset, delivery, rest @ ..] = segments.as_slice() else {
            return Err(WrongStructure);
        };
        if *cloud != policy.cloud_name {
            return Err(WrongCloud);
        }
        if *asset != "image" {
            return Err(WrongAssetType);
        }
        if *delivery != "upload" {
            return Err(WrongDeliveryType);
        }

        let rest = match rest {
            [first, tail @ ..] if is_version_segment(first) => tail,
            _ => rest,
        };
        let [folder, owner, file] = rest else {
            return Err(WrongStructure);
        };
        if *folder != AVATAR_FOLDER || *owner != policy.owner.to_string() {
            return Err(NotOwner);
        }

        let extension_allowed = file.rsplit_once('.').is_some_and(|(_, ext)| {
            ALLOWED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        });
        if !extension_allowed {
            return Err(BadExtension);
        }

        // Persist the normalized form that was validated, not the raw input.
        Ok(Self(url.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

fn is_version_segment(segment: &str) -> bool {
    segment
        .strip_prefix('v')
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOUD: &str = "demo";

    fn owner() -> Uuid {
        Uuid::from_u128(1)
    }
    fn other() -> Uuid {
        Uuid::from_u128(2)
    }
    fn policy() -> AvatarPolicy<'static> {
        AvatarPolicy {
            cloud_name: CLOUD,
            owner: owner(),
        }
    }
    fn good(file: &str) -> String {
        format!(
            "https://res.cloudinary.com/{CLOUD}/image/upload/v1700000000/avatars/{}/{file}",
            owner()
        )
    }

    #[test]
    fn accepts_owner_url_with_or_without_version() {
        let with_version = good("abc.jpg");
        let parsed = AvatarUrl::parse(&with_version, &policy()).unwrap();
        assert_eq!(parsed.as_str(), with_version);

        let no_version = format!(
            "https://res.cloudinary.com/{CLOUD}/image/upload/avatars/{}/abc.PNG",
            owner()
        );
        assert!(AvatarUrl::parse(&no_version, &policy()).is_ok());
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let raw = format!("  {}  ", good("abc.jpg"));
        let parsed = AvatarUrl::parse(&raw, &policy()).unwrap();
        assert_eq!(parsed.as_str(), good("abc.jpg"));
    }

    #[test]
    fn rejects_invalid_urls() {
        use AvatarUrlError::*;
        let me = owner();
        let cases: Vec<(String, AvatarUrlError)> = vec![
            (String::new(), Empty),
            ("   ".into(), Empty),
            (
                format!(
                    "https://res.cloudinary.com/{CLOUD}/image/upload/avatars/{me}/{}.jpg",
                    "a".repeat(MAX_LEN)
                ),
                TooLong,
            ),
            ("not a url".into(), Malformed),
            (good("abc.jpg").replace("https", "http"), NotHttps),
            // host tricks
            (
                good("abc.jpg").replace("res.cloudinary.com", "evilcloudinary.com"),
                WrongHost,
            ),
            (
                good("abc.jpg").replace("res.cloudinary.com", "api.cloudinary.com"),
                WrongHost,
            ),
            (
                good("abc.jpg").replace("res.cloudinary.com", "res.cloudinary.com@evil.com"),
                WrongHost,
            ),
            (
                good("abc.jpg").replace("res.cloudinary.com", "user:pw@res.cloudinary.com"),
                Malformed,
            ),
            (format!("{}?x=1", good("abc.jpg")), Malformed),
            // path structure
            (good("abc.jpg").replace(CLOUD, "someone-else"), WrongCloud),
            (
                good("abc.jpg").replace("/image/", "/video/"),
                WrongAssetType,
            ),
            (
                good("abc.jpg").replace("/upload/", "/fetch/"),
                WrongDeliveryType,
            ),
            (
                format!("https://res.cloudinary.com/{CLOUD}/image/upload/avatars/{me}"),
                WrongStructure,
            ),
            // ownership
            (
                good("abc.jpg").replace(&me.to_string(), &other().to_string()),
                NotOwner,
            ),
            (good("abc.jpg").replace("avatars", "portfolio"), NotOwner),
            // extension
            (good("abc.exe"), BadExtension),
            (good("abc"), BadExtension),
        ];

        for (input, expected) in cases {
            assert_eq!(
                AvatarUrl::parse(&input, &policy()).unwrap_err(),
                expected,
                "input: {input}"
            );
        }
    }

    #[test]
    fn validates_the_normalized_url_not_the_raw_input() {
        // The URL parser resolves ".." before we inspect the path.
        let escape = format!(
            "https://res.cloudinary.com/{CLOUD}/image/upload/avatars/{}/../{}/x.jpg",
            owner(),
            other()
        );
        assert_eq!(
            AvatarUrl::parse(&escape, &policy()).unwrap_err(),
            AvatarUrlError::NotOwner
        );

        let harmless = format!(
            "https://res.cloudinary.com/{CLOUD}/image/upload/avatars/{}/../{}/x.jpg",
            other(),
            owner()
        );
        let parsed = AvatarUrl::parse(&harmless, &policy()).unwrap();
        assert!(!parsed.as_str().contains(".."));
    }

    #[test]
    fn folder_matches_what_the_parser_expects() {
        assert_eq!(avatar_folder(owner()), format!("avatars/{}", owner()));
    }
}
