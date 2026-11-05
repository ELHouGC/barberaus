use std::collections::{BTreeSet, HashSet};

use crate::application::error::{AppError, Violation};
use crate::transport::dto::style_photos::{StyleCategory, StylePhotoResponse};

#[derive(Clone, Copy)]
struct CatalogueEntry {
    id: &'static str,
    src: &'static str,
    alt: &'static str,
    styles: &'static [&'static str],
}

const GENTS_TEXTURED_FADE: CatalogueEntry = CatalogueEntry {
    id: "gents-textured-fade",
    src: "https://images.unsplash.com/photo-1613772884837-87ba2082e2e3?auto=format&fit=crop&w=800&q=85",
    alt: "Textured fade haircut",
    styles: &["fade", "textured", "short"],
};
const GENTS_SHORT_CROP: CatalogueEntry = CatalogueEntry {
    id: "gents-short-crop",
    src: "https://images.unsplash.com/photo-1578390432942-d323db577792?auto=format&fit=crop&w=800&q=85",
    alt: "Short crop haircut",
    styles: &["crop", "short"],
};
const GENTS_CLASSIC_TAPER: CatalogueEntry = CatalogueEntry {
    id: "gents-classic-taper",
    src: "https://images.unsplash.com/photo-1611420745411-ed8fee30b8fb?auto=format&fit=crop&w=800&q=85",
    alt: "Classic tapered haircut",
    styles: &["taper", "classic", "short"],
};
const GENTS_CLEAN_FADE: CatalogueEntry = CatalogueEntry {
    id: "gents-clean-fade",
    src: "https://images.unsplash.com/photo-1486642100029-66db6e0a97e3?auto=format&fit=crop&w=800&q=85",
    alt: "Clean fade and beard",
    styles: &["fade", "beard"],
};
const GENTS_MODERN_CUT: CatalogueEntry = CatalogueEntry {
    id: "gents-modern-cut",
    src: "https://images.unsplash.com/photo-1599351431202-1e0f0137899a?auto=format&fit=crop&w=800&q=85",
    alt: "Modern haircut",
    styles: &["modern"],
};
const GENTS_CURLY_TOP: CatalogueEntry = CatalogueEntry {
    id: "gents-curly-top",
    src: "https://images.unsplash.com/photo-1621605815971-fbc98d665033?auto=format&fit=crop&w=800&q=85",
    alt: "Curly top haircut",
    styles: &["curly", "textured", "volume"],
};
const LADIES_BRAIDS: CatalogueEntry = CatalogueEntry {
    id: "ladies-braids",
    src: "https://images.unsplash.com/photo-1673470907547-1c0c6a996095?auto=format&fit=crop&w=800&q=85",
    alt: "Braided hairstyle",
    styles: &["braids"],
};
const LADIES_BLONDE_BOB: CatalogueEntry = CatalogueEntry {
    id: "ladies-blonde-bob",
    src: "https://images.unsplash.com/photo-1648157963892-fa90a04e278b?auto=format&fit=crop&w=800&q=85",
    alt: "Blonde bob hairstyle",
    styles: &["bob", "color-blonde"],
};
const LADIES_PIXIE: CatalogueEntry = CatalogueEntry {
    id: "ladies-pixie",
    src: "https://images.unsplash.com/photo-1653656120465-ae4eaa59ed33?auto=format&fit=crop&w=800&q=85",
    alt: "Pixie hairstyle",
    styles: &["pixie", "short"],
};
const LADIES_BUZZ: CatalogueEntry = CatalogueEntry {
    id: "ladies-buzz",
    src: "https://images.unsplash.com/photo-1647338064753-865bc227ebb7?auto=format&fit=crop&w=800&q=85",
    alt: "Buzz cut hairstyle",
    styles: &["buzz", "short"],
};
const LADIES_RED_LAYERS: CatalogueEntry = CatalogueEntry {
    id: "ladies-red-layers",
    src: "https://images.unsplash.com/photo-1706607599714-472e63e7f4e0?auto=format&fit=crop&w=800&q=85",
    alt: "Red layered hairstyle",
    styles: &["layers", "color-red"],
};
const LADIES_NATURAL_CURLS: CatalogueEntry = CatalogueEntry {
    id: "ladies-natural-curls",
    src: "https://images.unsplash.com/photo-1604898419914-4d580cf22fea?auto=format&fit=crop&w=800&q=85",
    alt: "Natural curly hairstyle",
    styles: &["curly", "volume"],
};
const ALL_LONG_LAYERS: CatalogueEntry = CatalogueEntry {
    id: "all-long-layers",
    src: "https://images.unsplash.com/photo-1644669236339-2aa66ff27920?auto=format&fit=crop&w=800&q=85",
    alt: "Long layered hairstyle inspiration",
    styles: &["long", "layers"],
};

static GENTS: [&CatalogueEntry; 6] = [
    &GENTS_TEXTURED_FADE,
    &GENTS_SHORT_CROP,
    &GENTS_CLASSIC_TAPER,
    &GENTS_CLEAN_FADE,
    &GENTS_MODERN_CUT,
    &GENTS_CURLY_TOP,
];
static LADIES: [&CatalogueEntry; 6] = [
    &LADIES_BRAIDS,
    &LADIES_BLONDE_BOB,
    &LADIES_PIXIE,
    &LADIES_BUZZ,
    &LADIES_RED_LAYERS,
    &LADIES_NATURAL_CURLS,
];
static ALL: [&CatalogueEntry; 6] = [
    &LADIES_BRAIDS,
    &GENTS_SHORT_CROP,
    &LADIES_PIXIE,
    &GENTS_CLASSIC_TAPER,
    &LADIES_BUZZ,
    &ALL_LONG_LAYERS,
];

#[derive(Debug, Default, Clone, Copy)]
pub struct StylePhotoCatalogService;

impl StylePhotoCatalogService {
    pub fn get_style_photos(category: StyleCategory) -> Vec<StylePhotoResponse> {
        let entries: &[&CatalogueEntry] = match category {
            StyleCategory::Gents => &GENTS,
            StyleCategory::Ladies => &LADIES,
            StyleCategory::All => &ALL,
        };

        entries
            .iter()
            .map(|entry| StylePhotoResponse {
                id: entry.id.to_string(),
                src: entry.src.to_string(),
                alt: entry.alt.to_string(),
            })
            .collect()
    }

    pub fn resolve_style_ids(selected_photo_ids: &[String]) -> Result<Vec<String>, AppError> {
        let mut styles = BTreeSet::new();
        let mut seen_photo_ids = HashSet::new();
        let mut violations = Vec::new();

        for (index, photo_id) in selected_photo_ids.iter().enumerate() {
            if !seen_photo_ids.insert(photo_id.as_str()) {
                violations.push(Violation {
                    pointer: format!("/selected_style_photo_ids/{index}"),
                    key: "validation.duplicate",
                });
                continue;
            }

            let entry = GENTS
                .iter()
                .chain(LADIES.iter())
                .chain(ALL.iter())
                .find(|entry| entry.id == photo_id.as_str());

            match entry {
                Some(entry) => styles.extend(entry.styles.iter().copied()),
                None => violations.push(Violation {
                    pointer: format!("/selected_style_photo_ids/{index}"),
                    key: "validation.unknown_style_photo_id",
                }),
            }
        }

        if violations.is_empty() {
            Ok(styles.into_iter().map(str::to_string).collect())
        } else {
            Err(AppError::Validation(violations))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ALL, GENTS, LADIES, StylePhotoCatalogService};
    use crate::application::error::AppError;
    use crate::transport::dto::style_photos::StyleCategory;
    use std::collections::HashSet;

    #[test]
    fn each_category_returns_six_entries() {
        for category in [
            StyleCategory::Gents,
            StyleCategory::Ladies,
            StyleCategory::All,
        ] {
            assert_eq!(
                StylePhotoCatalogService::get_style_photos(category).len(),
                6
            );
        }
    }

    #[test]
    fn reused_entries_are_identical_across_categories() {
        let gents = StylePhotoCatalogService::get_style_photos(StyleCategory::Gents);
        let ladies = StylePhotoCatalogService::get_style_photos(StyleCategory::Ladies);
        let all = StylePhotoCatalogService::get_style_photos(StyleCategory::All);

        for id in [
            "ladies-braids",
            "gents-short-crop",
            "ladies-pixie",
            "gents-classic-taper",
            "ladies-buzz",
        ] {
            let all_entry = all.iter().find(|entry| entry.id == id).unwrap();
            let category_entry = gents
                .iter()
                .chain(ladies.iter())
                .find(|entry| entry.id == id)
                .unwrap();
            assert_eq!(all_entry, category_entry);
        }
    }

    #[test]
    fn categories_have_no_duplicate_ids() {
        for category in [
            StyleCategory::Gents,
            StyleCategory::Ladies,
            StyleCategory::All,
        ] {
            let photos = StylePhotoCatalogService::get_style_photos(category);
            let unique_ids: HashSet<_> = photos.iter().map(|photo| &photo.id).collect();
            assert_eq!(unique_ids.len(), photos.len());
        }
    }

    #[test]
    fn every_unique_catalogue_entry_has_style_identifiers() {
        let mut entries = BTreeSet::new();
        for entry in GENTS.iter().chain(LADIES.iter()).chain(ALL.iter()) {
            if entries.insert(entry.id) {
                assert!(!entry.styles.is_empty(), "{} has no styles", entry.id);
            }
        }
        assert_eq!(entries.len(), 13);
    }

    #[test]
    fn resolution_deduplicates_and_is_deterministic() {
        let first = vec![
            "gents-textured-fade".to_string(),
            "gents-short-crop".to_string(),
            "ladies-pixie".to_string(),
        ];
        let reversed = first.iter().rev().cloned().collect::<Vec<_>>();

        let resolved = StylePhotoCatalogService::resolve_style_ids(&first).unwrap();
        assert_eq!(resolved, ["crop", "fade", "pixie", "short", "textured"]);
        assert_eq!(
            resolved,
            StylePhotoCatalogService::resolve_style_ids(&reversed).unwrap()
        );
    }

    #[test]
    fn empty_selection_resolves_to_no_styles() {
        assert!(
            StylePhotoCatalogService::resolve_style_ids(&[])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn unknown_photo_ids_use_validation_violations() {
        let selected = vec!["unknown-photo".to_string()];
        let Err(AppError::Validation(violations)) =
            StylePhotoCatalogService::resolve_style_ids(&selected)
        else {
            panic!("expected an existing validation error");
        };

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pointer, "/selected_style_photo_ids/0");
        assert_eq!(violations[0].key, "validation.unknown_style_photo_id");
    }

    #[test]
    fn duplicate_photo_ids_use_validation_violations() {
        let selected = vec![
            "gents-short-crop".to_string(),
            "gents-short-crop".to_string(),
        ];
        let Err(AppError::Validation(violations)) =
            StylePhotoCatalogService::resolve_style_ids(&selected)
        else {
            panic!("expected an existing validation error");
        };

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pointer, "/selected_style_photo_ids/1");
        assert_eq!(violations[0].key, "validation.duplicate");
    }

    #[test]
    fn public_response_contains_only_catalogue_fields() {
        let response = StylePhotoCatalogService::get_style_photos(StyleCategory::Gents);
        let serialized = serde_json::to_value(&response[0]).unwrap();
        let object = serialized.as_object().unwrap();

        assert_eq!(object.len(), 3);
        assert!(object.contains_key("id"));
        assert!(object.contains_key("src"));
        assert!(object.contains_key("alt"));
        assert!(!object.contains_key("styles"));
    }
}
