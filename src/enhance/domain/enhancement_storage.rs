use std::path::Path;

use crate::photo::domain::working_image::WorkingImage;

use super::enhancement::Enhancement;

/// Where the enhancements of photos are kept between two runs of the app.
pub trait EnhancementStorage {
    /// `Ok(None)` when nothing is stored for a photo of `size`; `Err` with
    /// the reason when what is stored cannot be used.
    fn stored_enhancement(
        &self,
        photo: &Path,
        size: [u32; 2],
    ) -> Result<Option<Enhancement>, String>;

    fn store_enhancement(&self, photo: &Path, enhancement: &Enhancement) -> Result<(), String>;

    /// `original` enhanced by what is stored for `photo`.
    fn stored_enhanced_image(
        &self,
        photo: &Path,
        original: &WorkingImage,
    ) -> Result<Option<WorkingImage>, String> {
        let size = [original.width(), original.height()];
        let enhancement = self.stored_enhancement(photo, size)?;
        Ok(enhancement.and_then(|enhancement| enhancement.enhanced(original)))
    }
}
