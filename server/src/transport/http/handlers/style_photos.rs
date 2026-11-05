use axum::{Json, extract::RawQuery};

use crate::{
    application::{error::AppError, services::style_photo_catalog::StylePhotoCatalogService},
    transport::{
        dto::style_photos::{StyleCategory, StylePhotoResponse},
        response::ApiResponse,
    },
};

pub async fn get_style_photos(
    RawQuery(query): RawQuery,
) -> Result<Json<ApiResponse<Vec<StylePhotoResponse>>>, AppError> {
    let category = StyleCategory::from_query(query.as_deref())?;
    let photos = StylePhotoCatalogService::get_style_photos(category);
    Ok(Json(ApiResponse::success(photos)))
}
