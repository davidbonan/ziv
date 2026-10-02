use std::fmt;

use serde::{Deserialize, Serialize};

use super::edit::Edit;

/// 2: masks. A reader of version 1 would drop them and overwrite the file.
/// 3: zone masks, which a reader of version 2 cannot read.
/// 4: enhancement intensity, which a reader of version 3 would drop.
pub const CURRENT_VERSION: u32 = 4;

#[derive(Serialize, Deserialize)]
struct EditDocument {
    version: u32,
    #[serde(flatten)]
    edit: Edit,
}

#[derive(Deserialize)]
struct VersionOnly {
    version: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentError {
    NotAnEditDocument(String),
    FromNewerZiv { version: u32 },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnEditDocument(reason) => write!(formatter, "it cannot be read ({reason})"),
            Self::FromNewerZiv { version } => write!(
                formatter,
                "it was written by a newer ziv (format {version}, this ziv reads up to {CURRENT_VERSION})"
            ),
        }
    }
}

pub fn edit_document(edit: &Edit) -> String {
    let document = EditDocument {
        version: CURRENT_VERSION,
        edit: edit.clone(),
    };
    serde_json::to_string_pretty(&document).expect("an edit is plain numbers")
}

pub fn edit_of_document(text: &str) -> Result<Edit, DocumentError> {
    let not_a_document =
        |error: serde_json::Error| DocumentError::NotAnEditDocument(error.to_string());
    let VersionOnly { version } = serde_json::from_str(text).map_err(not_a_document)?;
    if version > CURRENT_VERSION {
        return Err(DocumentError::FromNewerZiv { version });
    }
    let document: EditDocument = serde_json::from_str(text).map_err(not_a_document)?;
    Ok(document.edit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::develop::domain::adjustments::Adjustments;
    use std::sync::Arc;

    use crate::develop::domain::brush::{BrushMask, Stroke};
    use crate::develop::domain::coverage_image::CoverageImage;
    use crate::develop::domain::linear_gradient::LinearGradient;
    use crate::develop::domain::mask::{Mask, MaskShape};
    use crate::develop::domain::polygon::Polygon;
    use crate::develop::domain::radial_gradient::RadialGradient;
    use crate::develop::domain::rectangle::Rectangle;
    use crate::develop::domain::white_balance::WhiteBalance;
    use crate::develop::domain::zone::{Zone, ZoneMask};

    fn edited() -> Edit {
        Edit::from(Adjustments {
            white_balance: Some(WhiteBalance {
                temperature: 6200.0,
                tint: -8.0,
            }),
            exposure: 0.35,
            shadows: 40.0,
            ..Adjustments::default()
        })
    }

    #[test]
    fn an_edit_is_read_back_as_it_was_written() {
        assert_eq!(edit_of_document(&edit_document(&edited())), Ok(edited()));
    }

    #[test]
    fn a_missing_adjustment_is_at_its_default_and_an_unknown_one_is_ignored() {
        let document = r#"{ "version": 1, "exposure": 1.5, "clarity": 30 }"#;

        let expected = Edit::from(Adjustments {
            exposure: 1.5,
            ..Adjustments::default()
        });
        assert_eq!(edit_of_document(document), Ok(expected));
    }

    #[test]
    fn a_document_from_a_newer_ziv_is_refused() {
        let document = r#"{ "version": 5, "exposure": 1.5 }"#;

        assert_eq!(
            edit_of_document(document),
            Err(DocumentError::FromNewerZiv { version: 5 })
        );
    }

    fn one_of_each_shape() -> Vec<MaskShape> {
        let stroke = Stroke {
            points: vec![[0.1, 0.2], [0.3, 0.25]],
            radius: 0.05,
            feather: 40.0,
            flow: 80.0,
            is_erasing: true,
        };
        vec![
            MaskShape::LinearGradient(LinearGradient {
                full: [0.1, 0.2],
                none: [0.7, 0.6],
            }),
            MaskShape::RadialGradient(RadialGradient {
                centre: [0.5, 0.3],
                radii: [0.2, 0.1],
                rotation: 0.4,
                feather: 60.0,
            }),
            MaskShape::Rectangle(Rectangle {
                centre: [0.4, 0.4],
                half_size: [0.2, 0.1],
                feather: 10.0,
            }),
            MaskShape::Polygon(Polygon {
                corners: vec![[0.1, 0.1], [0.5, 0.2], [0.3, 0.6]],
                feather: 0.0,
            }),
            MaskShape::Brush(BrushMask {
                strokes: vec![stroke],
            }),
            MaskShape::Zone(ZoneMask {
                zone: Zone::Sky,
                coverage: Arc::new(
                    CoverageImage::new([3, 2], vec![0, 128, 255, 255, 64, 0]).unwrap(),
                ),
            }),
        ]
    }

    #[test]
    fn masks_are_read_back_as_they_were_written() {
        let masks = one_of_each_shape().into_iter().map(|shape| Mask {
            adjustments: edited().adjustments,
            is_inverted: true,
            is_hidden: true,
            shape,
        });
        let edit = Edit {
            masks: masks.collect(),
            enhancement_intensity: 60.0,
            ..edited()
        };

        let document = edit_document(&edit);

        assert!(document.contains(r#""version": 4"#));
        assert_eq!(edit_of_document(&document), Ok(edit));
    }

    #[test]
    fn a_version_2_document_keeps_its_masks() {
        let document = r#"{ "version": 2, "masks": [
            { "shape": { "linear_gradient": { "full": [0.1, 0.2], "none": [0.7, 0.6] } } }
        ] }"#;

        let edit = edit_of_document(document).unwrap();

        assert_eq!(edit.mask_names(), ["Linear gradient 1"]);
    }

    #[test]
    fn a_zone_mask_whose_coverage_is_not_an_image_is_refused() {
        let document = r#"{ "version": 3, "masks": [
            { "shape": { "zone": { "zone": "sky", "coverage": "bm90IGFuIGltYWdl" } } }
        ] }"#;

        assert!(matches!(
            edit_of_document(document),
            Err(DocumentError::NotAnEditDocument(_))
        ));
    }

    #[test]
    fn a_version_1_document_is_an_edit_without_masks() {
        let document = r#"{ "version": 1, "exposure": 1.5 }"#;

        let edit = edit_of_document(document).unwrap();

        assert_eq!((edit.adjustments.exposure, edit.masks.len()), (1.5, 0));
        assert_eq!(edit.enhancement_intensity, 0.0);
    }

    #[test]
    fn text_that_is_not_a_document_is_refused() {
        for text in ["", "not json", r#"{ "exposure": 1.5 }"#] {
            assert!(matches!(
                edit_of_document(text),
                Err(DocumentError::NotAnEditDocument(_))
            ));
        }
    }
}
