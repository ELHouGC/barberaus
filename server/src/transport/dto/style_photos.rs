use crate::application::error::{AppError, Violation};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleCategory {
    Gents,
    Ladies,
    All,
}

impl StyleCategory {
    pub fn from_query(query: Option<&str>) -> Result<Self, AppError> {
        let categories: Vec<String> = query
            .into_iter()
            .flat_map(|query| url::form_urlencoded::parse(query.as_bytes()))
            .filter(|(key, _)| key == "category")
            .map(|(_, value)| value.into_owned())
            .collect();

        match categories.as_slice() {
            [] => Err(category_violation("validation.required")),
            [category] => match category.as_str() {
                "gents" => Ok(Self::Gents),
                "ladies" => Ok(Self::Ladies),
                "all" => Ok(Self::All),
                _ => Err(category_violation("validation.unsupported_category")),
            },
            _ => Err(category_violation("validation.duplicate")),
        }
    }
}

fn category_violation(key: &'static str) -> AppError {
    AppError::Validation(vec![Violation {
        pointer: "/category".to_string(),
        key,
    }])
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StylePhotoResponse {
    pub id: String,
    pub src: String,
    pub alt: String,
}

#[cfg(test)]
mod tests {
    use super::{AppError, StyleCategory};

    #[test]
    fn parses_supported_categories() {
        assert_eq!(
            StyleCategory::from_query(Some("category=gents")).unwrap(),
            StyleCategory::Gents
        );
        assert_eq!(
            StyleCategory::from_query(Some("category=ladies")).unwrap(),
            StyleCategory::Ladies
        );
        assert_eq!(
            StyleCategory::from_query(Some("category=all")).unwrap(),
            StyleCategory::All
        );
    }

    #[test]
    fn unsupported_category_is_a_validation_error() {
        assert_validation_error(
            StyleCategory::from_query(Some("category=children")),
            "validation.unsupported_category",
        );
    }

    #[test]
    fn missing_category_is_a_validation_error() {
        assert_validation_error(StyleCategory::from_query(None), "validation.required");
        assert_validation_error(
            StyleCategory::from_query(Some("other=value")),
            "validation.required",
        );
    }

    fn assert_validation_error(
        result: Result<StyleCategory, AppError>,
        expected_key: &'static str,
    ) {
        let Err(AppError::Validation(violations)) = result else {
            panic!("expected a standardized validation error");
        };
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pointer, "/category");
        assert_eq!(violations[0].key, expected_key);
    }
}
