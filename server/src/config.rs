#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string()),
        }
    }
}
