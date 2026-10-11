use chrono::{DateTime, Utc};




pub fn default_registered_at() -> chrono::DateTime<chrono::Utc> {
    DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now)
}
