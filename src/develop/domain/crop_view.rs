use crate::photo::domain::picture_region::PictureRegion;
use crate::viewport::domain::fit::fit_size;

use super::framing::{Framing, PicturePoint};

/// The whole picture as crop mode lays it out: turned as its framing turns
/// it, so that the frame is upright, at fit in the middle of an area.
/// Positions on screen are in pixels from the top-left of that area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropView {
    /// Screen pixels per pixel of the picture.
    scale: f32,
    picture: [f32; 2],
    /// The upright rectangle of the screen holding the turned picture.
    bounds_min: [f32; 2],
    bounds_size: [f32; 2],
    /// Where a step right and a step down on screen lead in the picture.
    across: [f32; 2],
    down: [f32; 2],
}

impl CropView {
    pub fn fitting(picture: [u32; 2], framing: &Framing, area: [f32; 2]) -> Self {
        let [across, down] = framing.steps_in_picture();
        let [width, height] = picture.map(|side| side as f32);
        let turned = [across, down].map(|step| width * step[0].abs() + height * step[1].abs());
        // Whole-pixel size and margin: one texel of the render per screen pixel.
        let bounds_size =
            fit_size(turned.map(|side| side.ceil() as u32), area).map(|side| side.round().max(1.0));
        Self {
            scale: bounds_size[0] / turned[0],
            picture: [width, height],
            bounds_min: [0, 1].map(|axis| ((area[axis] - bounds_size[axis]) / 2.0).floor()),
            bounds_size,
            across,
            down,
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Where the upright rectangle holding the turned picture starts on screen.
    pub fn bounds_min(&self) -> [f32; 2] {
        self.bounds_min
    }

    pub fn bounds_size(&self) -> [f32; 2] {
        self.bounds_size
    }

    fn middle(&self) -> [f32; 2] {
        [0, 1].map(|axis| self.bounds_min[axis] + self.bounds_size[axis] / 2.0)
    }

    pub fn on_screen(&self, point: PicturePoint) -> [f32; 2] {
        let from_centre = [0, 1].map(|axis| point[axis] - self.picture[axis] / 2.0);
        let middle = self.middle();
        let along = [self.across, self.down]
            .map(|side| from_centre[0] * side[0] + from_centre[1] * side[1]);
        [0, 1].map(|axis| middle[axis] + along[axis] * self.scale)
    }

    pub fn in_picture(&self, position: [f32; 2]) -> PicturePoint {
        let middle = self.middle();
        let [across, down] = [0, 1].map(|axis| (position[axis] - middle[axis]) / self.scale);
        [0, 1].map(|axis| {
            self.picture[axis] / 2.0 + across * self.across[axis] + down * self.down[axis]
        })
    }

    /// What the upright rectangle holding the turned picture shows of it.
    pub fn region(&self) -> PictureRegion {
        let in_shares = |point: PicturePoint| [0, 1].map(|axis| point[axis] / self.picture[axis]);
        let side_in_shares = |side: [f32; 2], length: f32| {
            [0, 1].map(|axis| side[axis] * length / self.scale / self.picture[axis])
        };
        PictureRegion {
            origin: in_shares(self.in_picture(self.bounds_min)),
            across: side_in_shares(self.across, self.bounds_size[0]),
            down: side_in_shares(self.down, self.bounds_size[1]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::develop::domain::framing::Turn;

    fn straightened(angle: f32) -> Framing {
        Framing {
            angle,
            ..Framing::default()
        }
    }

    fn assert_close(actual: [f32; 2], expected: [f32; 2]) {
        let is_close = [0, 1]
            .iter()
            .all(|axis| (actual[*axis] - expected[*axis]).abs() < 0.01);
        assert!(is_close, "{actual:?} differs from {expected:?}");
    }

    #[test]
    fn the_picture_is_fitted_in_the_middle_of_the_area() {
        let view = CropView::fitting([4000, 2000], &straightened(0.0), [800.0, 600.0]);

        assert_eq!(view.bounds_size(), [800.0, 400.0]);
        assert_eq!(view.on_screen([0.0, 0.0]), [0.0, 100.0]);
        assert_eq!(view.on_screen([4000.0, 2000.0]), [800.0, 500.0]);
        assert_eq!(view.region(), PictureRegion::WHOLE);
    }

    #[test]
    fn a_screen_position_is_found_back_in_the_picture() {
        let view = CropView::fitting([4000, 2000], &straightened(20.0), [800.0, 600.0]);

        assert_close(
            view.in_picture(view.on_screen([1000.0, 300.0])),
            [1000.0, 300.0],
        );
        let [across, down] = view.on_screen([2000.0, 1000.0]);
        assert!((across - 400.0).abs() <= 0.5 && (down - 300.0).abs() <= 0.5);
    }

    #[test]
    fn a_turned_picture_is_shown_clockwise_and_whole() {
        let view = CropView::fitting([400, 200], &straightened(45.0), [800.0, 600.0]);

        let right = view.on_screen([400.0, 100.0]);
        let middle = view.on_screen([200.0, 100.0]);
        assert!(right[0] > middle[0] && right[1] > middle[1]);
        assert_close([right[0] - middle[0], 0.0], [right[1] - middle[1], 0.0]);
        for corner in [[0.0, 0.0], [400.0, 0.0], [400.0, 200.0], [0.0, 200.0]] {
            let [across, down] = view.on_screen(corner);
            let min = view.bounds_min();
            let size = view.bounds_size();
            assert!(across >= min[0] - 1.0 && across <= min[0] + size[0] + 1.0);
            assert!(down >= min[1] - 1.0 && down <= min[1] + size[1] + 1.0);
        }
    }

    #[test]
    fn the_region_of_a_turned_picture_reaches_outside_it() {
        let view = CropView::fitting([400, 200], &straightened(30.0), [800.0, 600.0]);

        let region = view.region();

        assert_close(region.at([0.5, 0.5]), [0.5, 0.5]);
        assert!(region.origin[0] < 0.0 || region.origin[1] < 0.0);
    }

    #[test]
    fn a_picture_turned_right_is_laid_out_on_its_side() {
        let framing = Framing {
            turn: Turn::default().turned_right(),
            ..Framing::default()
        };

        let view = CropView::fitting([4000, 2000], &framing, [800.0, 600.0]);

        assert_eq!(view.bounds_size(), [300.0, 600.0]);
        assert_close(view.on_screen([0.0, 0.0]), [550.0, 0.0]);
        assert_close(view.on_screen([4000.0, 0.0]), [550.0, 600.0]);
        assert_close(view.in_picture([250.0, 0.0]), [0.0, 2000.0]);
    }

    #[test]
    fn a_mirrored_picture_is_laid_out_the_other_way_round() {
        let framing = Framing {
            turn: Turn::default().flipped_horizontally(),
            ..Framing::default()
        };

        let view = CropView::fitting([4000, 2000], &framing, [800.0, 600.0]);

        assert_close(view.on_screen([0.0, 0.0]), [800.0, 100.0]);
        assert_close(view.region().at([0.0, 0.0]), [1.0, 0.0]);
    }
}
