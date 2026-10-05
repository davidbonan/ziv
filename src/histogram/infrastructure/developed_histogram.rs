use crate::develop::domain::development::Development;
use crate::develop::domain::framing::Framing;
use crate::engine::infrastructure::display_stage::DisplayRequest;
use crate::engine::infrastructure::engine::Engine;
use crate::engine::infrastructure::source_texture::SourceTexture;
use crate::histogram::domain::histogram::{Histogram, sample_size};

/// What a histogram is counted on: a photo developed and framed.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CountedPhoto {
    pub development: Development,
    pub framing: Framing,
}

/// The histogram of the framed photo developed, counted on a reduced render.
pub fn histogram_of(
    engine: &Engine,
    source: &SourceTexture,
    photo: &CountedPhoto,
) -> Result<Histogram, wgpu::BufferAsyncError> {
    let picture = source.size();
    let request = DisplayRequest {
        region: photo.framing.region(picture),
        development: photo.development.clone(),
        ..DisplayRequest::whole_source(sample_size(photo.framing.framed_size(picture)))
    };
    let reduced = engine.render_pixels(source, &request)?;
    Ok(Histogram::of_display_pixels(&reduced.rgba))
}

struct Counted {
    photo: CountedPhoto,
    is_enhanced: bool,
    histogram: Option<Histogram>,
}

/// The histogram of one photo on the GPU, counted again when its development changes.
#[derive(Default)]
pub struct DevelopedHistogram {
    counted: Option<Box<Counted>>,
}

impl DevelopedHistogram {
    pub fn latest(&self) -> Option<&Histogram> {
        self.counted.as_ref()?.histogram.as_ref()
    }

    /// Counts `source` developed and framed, unless the latest count already
    /// is that. Whether it counted.
    pub fn follow(
        &mut self,
        engine: &Engine,
        source: &SourceTexture,
        photo: &CountedPhoto,
    ) -> bool {
        let is_enhanced = source.is_enhanced();
        let is_latest =
            |counted: &Counted| counted.photo == *photo && counted.is_enhanced == is_enhanced;
        if self.counted.as_deref().is_some_and(is_latest) {
            return false;
        }
        self.counted = Some(Box::new(Counted {
            photo: photo.clone(),
            is_enhanced,
            histogram: histogram_of(engine, source, photo).ok(),
        }));
        true
    }
}
