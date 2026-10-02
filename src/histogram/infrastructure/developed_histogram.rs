use crate::develop::domain::development::Development;
use crate::engine::infrastructure::engine::Engine;
use crate::engine::infrastructure::source_texture::SourceTexture;
use crate::histogram::domain::histogram::{Histogram, sample_size};

/// The histogram of the whole photo developed, counted on a reduced render.
pub fn histogram_of(
    engine: &Engine,
    source: &SourceTexture,
    development: &Development,
) -> Result<Histogram, wgpu::BufferAsyncError> {
    let reduced = engine.render_pixels(source, development, sample_size(source.size()))?;
    Ok(Histogram::of_display_pixels(&reduced.rgba))
}

struct Counted {
    development: Development,
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

    /// Counts `source` developed, unless the latest count already is that.
    /// Whether it counted.
    pub fn follow(
        &mut self,
        engine: &Engine,
        source: &SourceTexture,
        development: &Development,
    ) -> bool {
        let is_enhanced = source.is_enhanced();
        let is_latest = |counted: &Counted| {
            counted.development == *development && counted.is_enhanced == is_enhanced
        };
        if self.counted.as_deref().is_some_and(is_latest) {
            return false;
        }
        self.counted = Some(Box::new(Counted {
            development: development.clone(),
            is_enhanced,
            histogram: histogram_of(engine, source, development).ok(),
        }));
        true
    }
}
