use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Clean application-layer structure containing only what the business logic needs.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct CropCoordinates {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct AvatarUpload {
    pub user_id: Uuid,
    pub avatar_url: String,
    pub crop_coordinates: CropCoordinates,
}
