use image::DynamicImage;
use image::metadata::Orientation;

use crate::photo::domain::thumbnail::{Thumbnail, thumbnail_size};

/// Thumbnail of an already display-encoded picture stored with `orientation`.
pub fn thumbnail_of_encoded(encoded: DynamicImage, orientation: Orientation) -> Thumbnail {
    let [width, height] = thumbnail_size(encoded.width(), encoded.height());
    let mut reduced = encoded.thumbnail_exact(width, height);
    reduced.apply_orientation(orientation);
    let reduced = reduced.into_rgba8();
    Thumbnail {
        width: reduced.width(),
        height: reduced.height(),
        rgba: reduced.into_raw(),
    }
}
