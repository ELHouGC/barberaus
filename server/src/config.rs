

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppEnvironment {
    Development,
    Production,
}

impl AppEnvironment {
    pub fn from_env() -> Self {
        match std::env::var("ENV").as_deref() {
            Ok("development") => Self::Development,
            Ok("production") => Self::Production,
            _ => panic!(
                "ENV must be set to either 'development' or 'production'"
            ),
        }
    }
}
#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub environment: AppEnvironment,
}

impl Config {
    pub fn from_env() -> Self {
        let jwt_secret =
        std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        assert!(
            !jwt_secret.trim().is_empty(),
            "JWT_SECRET must not be empty"
        );

        assert!(
            jwt_secret.as_bytes().len() >= 32,
            "JWT_SECRET must be at least 32 bytes for HS256"
        );

        Self { 
            jwt_secret ,
            environment: AppEnvironment::from_env(),
        }
    }
}
