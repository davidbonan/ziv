use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::adjustments::Adjustments;
use super::brush::BrushMask;
use super::coverage_image::CoverageImage;
use super::linear_gradient::{LinearGradient, NONE_HANDLE};
use super::polygon::Polygon;
use super::radial_gradient::{self, DEFAULT_FEATHER, RadialGradient};
use super::rectangle::{self, Rectangle};
use super::zone::{Zone, ZoneMask};

/// A place on the photo, in pixels divided by the photo's long edge: the same
/// at any render size, and a circle stays a circle.
pub type PhotoPoint = [f32; 2];

/// The bottom-right corner of a photo of `size` pixels.
pub fn photo_extent(size: [u32; 2]) -> PhotoPoint {
    let long_edge = size[0].max(size[1]) as f32;
    size.map(|side| side as f32 / long_edge)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskKind {
    LinearGradient,
    RadialGradient,
    Rectangle,
    Polygon,
    Brush,
    Zone(Zone),
}

impl MaskKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::LinearGradient => "Linear gradient",
            Self::RadialGradient => "Radial gradient",
            Self::Rectangle => "Rectangle",
            Self::Polygon => "Polygon",
            Self::Brush => "Brush",
            Self::Zone(zone) => zone.name(),
        }
    }

    /// The feather a mask of this kind is drawn with.
    pub fn default_feather(self) -> f32 {
        match self {
            Self::RadialGradient => DEFAULT_FEATHER,
            Self::LinearGradient
            | Self::Rectangle
            | Self::Polygon
            | Self::Brush
            | Self::Zone(_) => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskShape {
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    Rectangle(Rectangle),
    Polygon(Polygon),
    Brush(BrushMask),
    Zone(ZoneMask),
}

/// What the coverage image of a mask rendered from one is made of.
pub enum CoverageSource<'a> {
    Strokes(&'a BrushMask),
    Image(&'a Arc<CoverageImage>),
}

impl MaskShape {
    pub fn kind(&self) -> MaskKind {
        match self {
            Self::LinearGradient(_) => MaskKind::LinearGradient,
            Self::RadialGradient(_) => MaskKind::RadialGradient,
            Self::Rectangle(_) => MaskKind::Rectangle,
            Self::Polygon(_) => MaskKind::Polygon,
            Self::Brush(_) => MaskKind::Brush,
            Self::Zone(detected) => MaskKind::Zone(detected.zone),
        }
    }

    /// `None` for a shape whose coverage is computed from its geometry.
    pub fn coverage_source(&self) -> Option<CoverageSource<'_>> {
        match self {
            Self::Brush(brush) => Some(CoverageSource::Strokes(brush)),
            Self::Zone(detected) => Some(CoverageSource::Image(&detected.coverage)),
            Self::LinearGradient(_)
            | Self::RadialGradient(_)
            | Self::Rectangle(_)
            | Self::Polygon(_) => None,
        }
    }

    /// A shape of `kind` born at `point`, and the handle a drag from there
    /// pulls to draw it. `None` for a kind that is not drawn by one drag.
    pub fn drawn_from(kind: MaskKind, point: PhotoPoint) -> Option<(Self, usize)> {
        match kind {
            MaskKind::LinearGradient => Some((
                Self::LinearGradient(LinearGradient::starting_at(point)),
                NONE_HANDLE,
            )),
            MaskKind::RadialGradient => Some((
                Self::RadialGradient(RadialGradient::starting_at(point)),
                radial_gradient::DRAWING_HANDLE,
            )),
            MaskKind::Rectangle => Some((
                Self::Rectangle(Rectangle::starting_at(point)),
                rectangle::DRAWING_HANDLE,
            )),
            MaskKind::Polygon | MaskKind::Brush | MaskKind::Zone(_) => None,
        }
    }

    /// The points the shape is reshaped by.
    pub fn handles(&self) -> Vec<PhotoPoint> {
        match self {
            Self::LinearGradient(gradient) => gradient.handles().to_vec(),
            Self::RadialGradient(gradient) => gradient.handles().to_vec(),
            Self::Rectangle(rectangle) => rectangle.handles().to_vec(),
            Self::Polygon(polygon) => polygon.handles(),
            Self::Brush(_) | Self::Zone(_) => Vec::new(),
        }
    }

    pub fn with_handle_at(&self, handle: usize, point: PhotoPoint) -> Self {
        match self {
            Self::LinearGradient(gradient) => {
                Self::LinearGradient(gradient.with_handle_at(handle, point))
            }
            Self::RadialGradient(gradient) => {
                Self::RadialGradient(gradient.with_handle_at(handle, point))
            }
            Self::Rectangle(rectangle) => Self::Rectangle(rectangle.with_handle_at(handle, point)),
            Self::Polygon(polygon) => Self::Polygon(polygon.with_handle_at(handle, point)),
            Self::Brush(_) | Self::Zone(_) => self.clone(),
        }
    }

    /// `None` for a shape that has no feather to set.
    pub fn feather(&self) -> Option<f32> {
        match self {
            Self::LinearGradient(_) | Self::Brush(_) | Self::Zone(_) => None,
            Self::RadialGradient(gradient) => Some(gradient.feather),
            Self::Rectangle(rectangle) => Some(rectangle.feather),
            Self::Polygon(polygon) => Some(polygon.feather),
        }
    }

    pub fn with_feather(&self, feather: f32) -> Self {
        let mut feathered = self.clone();
        match &mut feathered {
            Self::LinearGradient(_) | Self::Brush(_) | Self::Zone(_) => {}
            Self::RadialGradient(gradient) => gradient.feather = feather,
            Self::Rectangle(rectangle) => rectangle.feather = feather,
            Self::Polygon(polygon) => polygon.feather = feather,
        }
        feathered
    }

    pub fn coverage(&self, point: PhotoPoint) -> f32 {
        match self {
            Self::LinearGradient(gradient) => gradient.coverage(point),
            Self::RadialGradient(gradient) => gradient.coverage(point),
            Self::Rectangle(rectangle) => rectangle.coverage(point),
            Self::Polygon(polygon) => polygon.coverage(point),
            Self::Brush(brush) => brush.coverage(point),
            Self::Zone(detected) => detected.coverage.coverage(point),
        }
    }
}

/// A region of the photo carrying its own adjustments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mask {
    pub shape: MaskShape,
    #[serde(default)]
    pub adjustments: Adjustments,
    #[serde(default)]
    pub is_inverted: bool,
    #[serde(default)]
    pub is_hidden: bool,
}

impl Mask {
    pub fn of(shape: MaskShape) -> Self {
        Self {
            shape,
            adjustments: Adjustments::default(),
            is_inverted: false,
            is_hidden: false,
        }
    }

    pub fn has_adjustments(&self) -> bool {
        self.adjustments != Adjustments::default()
    }

    /// How much the mask applies at `point`: 0 not at all, 1 fully.
    pub fn coverage(&self, point: PhotoPoint) -> f32 {
        let covered = self.shape.coverage(point);
        match self.is_inverted {
            true => 1.0 - covered,
            false => covered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn left_to_right() -> Mask {
        Mask::of(MaskShape::LinearGradient(LinearGradient {
            full: [0.25, 0.5],
            none: [0.75, 0.5],
        }))
    }

    #[test]
    fn inverted_mask_covers_what_it_left() {
        let inverted = Mask {
            is_inverted: true,
            ..left_to_right()
        };

        assert_eq!(left_to_right().coverage([0.1, 0.5]), 1.0);
        assert_eq!(inverted.coverage([0.1, 0.5]), 0.0);
        assert_eq!(inverted.coverage([0.9, 0.5]), 1.0);
    }

    #[test]
    fn photo_extent_is_one_along_the_long_edge() {
        assert_eq!(photo_extent([6000, 4000]), [1.0, 4000.0 / 6000.0]);
        assert_eq!(photo_extent([4000, 6000]), [4000.0 / 6000.0, 1.0]);
    }
}
