use super::fit::fit_size;

pub const MAX_SCALE: f32 = 8.0;
pub const ACTUAL_SIZE: f32 = 1.0;

/// What a view is laid out in: the photo and the screen area showing it, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub photo: [u32; 2],
    pub size: [f32; 2],
}

impl Viewport {
    fn photo_size(&self) -> [f32; 2] {
        self.photo.map(|side| side as f32)
    }

    /// Screen pixels per photo pixel when the whole photo is visible.
    pub fn fit_scale(&self) -> f32 {
        fit_size(self.photo, self.size)[0] / self.photo_size()[0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Zoom {
    Fit,
    Scale(f32),
}

/// Which part of the photo is on screen. Positions are in pixels: `center` in
/// the photo, pointers and pans on screen relative to the viewport's top-left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    zoom: Zoom,
    center: [f32; 2],
}

/// Where the visible part of the photo goes, both in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Screen pixels per photo pixel.
    pub scale: f32,
    pub screen_min: [f32; 2],
    pub screen_size: [f32; 2],
    pub photo_min: [f32; 2],
    pub photo_size: [f32; 2],
}

impl View {
    pub fn fit() -> Self {
        Self {
            zoom: Zoom::Fit,
            center: [0.0; 2],
        }
    }

    pub fn is_fit(&self) -> bool {
        self.zoom == Zoom::Fit
    }

    pub fn scale(&self, viewport: &Viewport) -> f32 {
        match self.zoom {
            Zoom::Fit => viewport.fit_scale(),
            Zoom::Scale(scale) => scale,
        }
    }

    /// The photo position shown at `pointer`.
    pub fn photo_position_at(&self, viewport: &Viewport, pointer: [f32; 2]) -> [f32; 2] {
        let view = self.constrained_to(viewport);
        let scale = view.scale(viewport);
        [0, 1].map(|axis| view.center[axis] + (pointer[axis] - viewport.size[axis] / 2.0) / scale)
    }

    /// The same view made valid for `viewport`: zoom within fit…800 %, no empty
    /// margin on an axis where the photo overflows, centered on the others.
    pub fn constrained_to(self, viewport: &Viewport) -> Self {
        let fit_scale = viewport.fit_scale();
        let scale = match self.zoom {
            Zoom::Scale(scale) if scale > fit_scale => scale.min(MAX_SCALE),
            _ => {
                return Self {
                    zoom: Zoom::Fit,
                    center: viewport.photo_size().map(|side| side / 2.0),
                };
            }
        };
        let photo = viewport.photo_size();
        let center = [0, 1].map(|axis| {
            let half_visible = viewport.size[axis] / scale / 2.0;
            if half_visible * 2.0 >= photo[axis] {
                return photo[axis] / 2.0;
            }
            self.center[axis].clamp(half_visible, photo[axis] - half_visible)
        });
        Self {
            zoom: Zoom::Scale(scale),
            center,
        }
    }

    fn scaled_keeping(self, viewport: &Viewport, pointer: [f32; 2], scale: f32) -> Self {
        let anchor = self.photo_position_at(viewport, pointer);
        let center =
            [0, 1].map(|axis| anchor[axis] - (pointer[axis] - viewport.size[axis] / 2.0) / scale);
        Self {
            zoom: Zoom::Scale(scale),
            center,
        }
        .constrained_to(viewport)
    }

    /// Zooms by `factor`, the photo position under `pointer` staying under it.
    pub fn zoomed_around(self, viewport: &Viewport, pointer: [f32; 2], factor: f32) -> Self {
        let scale = (self.scale(viewport) * factor).min(MAX_SCALE);
        self.scaled_keeping(viewport, pointer, scale)
    }

    /// Fit → 100 % centered on the photo position under `pointer`; anything else → fit.
    pub fn toggled_at(self, viewport: &Viewport, pointer: [f32; 2]) -> Self {
        if !self.constrained_to(viewport).is_fit() {
            return Self::fit();
        }
        Self {
            zoom: Zoom::Scale(ACTUAL_SIZE),
            center: self.photo_position_at(viewport, pointer),
        }
        .constrained_to(viewport)
    }

    /// 100 %, keeping what is at the middle of the viewport.
    pub fn at_actual_size(self, viewport: &Viewport) -> Self {
        let middle = viewport.size.map(|side| side / 2.0);
        self.scaled_keeping(viewport, middle, ACTUAL_SIZE)
    }

    /// Moves the photo by `drag` screen pixels.
    pub fn panned_by(self, viewport: &Viewport, drag: [f32; 2]) -> Self {
        let view = self.constrained_to(viewport);
        let scale = view.scale(viewport);
        Self {
            zoom: view.zoom,
            center: [0, 1].map(|axis| view.center[axis] - drag[axis] / scale),
        }
        .constrained_to(viewport)
    }

    /// The same part of the photo on screen once the photo is shown `factor`
    /// times as large in pixels: a picture replaced by a bigger one of the same photo.
    pub fn for_photo_scaled_by(self, factor: f32) -> Self {
        let zoom = match self.zoom {
            Zoom::Fit => Zoom::Fit,
            Zoom::Scale(scale) => Zoom::Scale(scale / factor),
        };
        Self {
            zoom,
            center: self.center.map(|position| position * factor),
        }
    }

    pub fn placement(&self, viewport: &Viewport) -> Placement {
        let view = self.constrained_to(viewport);
        let scale = view.scale(viewport);
        let photo = viewport.photo_size();
        let axes = [0, 1].map(|axis| {
            AxisPlacement::new(photo[axis], viewport.size[axis], scale, view.center[axis])
        });
        Placement {
            scale,
            screen_min: axes.map(|axis| axis.screen_min),
            screen_size: axes.map(|axis| axis.screen_size),
            photo_min: axes.map(|axis| axis.photo_min),
            photo_size: axes.map(|axis| axis.photo_size),
        }
    }
}

#[derive(Clone, Copy)]
struct AxisPlacement {
    screen_min: f32,
    screen_size: f32,
    photo_min: f32,
    photo_size: f32,
}

impl AxisPlacement {
    fn new(photo: f32, viewport: f32, scale: f32, center: f32) -> Self {
        let scaled = photo * scale;
        if scaled > viewport {
            let visible = viewport / scale;
            return Self {
                screen_min: 0.0,
                screen_size: viewport,
                photo_min: center - visible / 2.0,
                photo_size: visible,
            };
        }
        // Whole-pixel size and margin: one texel of the render per screen pixel.
        let screen_size = scaled.round().max(1.0);
        Self {
            screen_min: ((viewport - screen_size) / 2.0).floor(),
            screen_size,
            photo_min: 0.0,
            photo_size: photo,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Viewport = Viewport {
        photo: [4000, 2000],
        size: [800.0, 600.0],
    };
    const MIDDLE: [f32; 2] = [400.0, 300.0];

    fn assert_close(actual: [f32; 2], expected: [f32; 2]) {
        assert!(
            (actual[0] - expected[0]).abs() < 0.01 && (actual[1] - expected[1]).abs() < 0.01,
            "{actual:?} differs from {expected:?}"
        );
    }

    #[test]
    fn a_bigger_picture_of_the_photo_keeps_the_same_part_on_screen() {
        let small = Viewport {
            photo: [1000, 500],
            ..VIEWPORT
        };
        let zoomed = View::fit().zoomed_around(&small, [100.0, 100.0], 3.0);

        let replaced = zoomed.for_photo_scaled_by(4.0);

        let before = zoomed.placement(&small);
        let after = replaced.placement(&VIEWPORT);
        assert_close(
            after.photo_min,
            before.photo_min.map(|position| position * 4.0),
        );
        assert_close(after.screen_size, before.screen_size);
        assert!((after.scale - before.scale / 4.0).abs() < 1e-6);
    }

    #[test]
    fn fit_stays_fit_when_the_picture_is_replaced() {
        assert!(View::fit().for_photo_scaled_by(4.0).is_fit());
    }

    #[test]
    fn fit_shows_the_whole_photo_centered() {
        let placement = View::fit().placement(&VIEWPORT);

        assert_eq!(placement.screen_min, [0.0, 100.0]);
        assert_eq!(placement.screen_size, [800.0, 400.0]);
        assert_eq!(placement.photo_min, [0.0, 0.0]);
        assert_eq!(placement.photo_size, [4000.0, 2000.0]);
    }

    #[test]
    fn fit_follows_the_viewport_when_it_is_resized() {
        let resized = Viewport {
            size: [400.0, 600.0],
            ..VIEWPORT
        };

        let placement = View::fit().placement(&resized);

        assert_eq!(placement.screen_size, [400.0, 200.0]);
    }

    #[test]
    fn zooming_keeps_the_photo_position_under_the_pointer() {
        let pointer = [600.0, 250.0];
        let before = View::fit().photo_position_at(&VIEWPORT, pointer);

        let zoomed = View::fit().zoomed_around(&VIEWPORT, pointer, 2.0);

        assert!((zoomed.scale(&VIEWPORT) - 0.4).abs() < 1e-6);
        assert_close(zoomed.photo_position_at(&VIEWPORT, pointer), before);
    }

    #[test]
    fn zooming_out_stops_at_fit() {
        let zoomed_in = View::fit().zoomed_around(&VIEWPORT, MIDDLE, 3.0);

        let zoomed_out = zoomed_in.zoomed_around(&VIEWPORT, MIDDLE, 0.01);

        assert!(zoomed_out.is_fit());
    }

    #[test]
    fn zooming_in_stops_at_800_percent() {
        let zoomed = View::fit().zoomed_around(&VIEWPORT, MIDDLE, 1000.0);

        assert_eq!(zoomed.scale(&VIEWPORT), MAX_SCALE);
    }

    #[test]
    fn toggling_from_fit_goes_to_actual_size_centered_on_the_pointer() {
        let pointer = [500.0, 350.0];
        let clicked = View::fit().photo_position_at(&VIEWPORT, pointer);

        let toggled = View::fit().toggled_at(&VIEWPORT, pointer);

        assert_eq!(toggled.scale(&VIEWPORT), ACTUAL_SIZE);
        assert_close(toggled.photo_position_at(&VIEWPORT, MIDDLE), clicked);
    }

    #[test]
    fn toggling_when_zoomed_goes_back_to_fit() {
        let zoomed = View::fit().zoomed_around(&VIEWPORT, MIDDLE, 3.0);

        assert!(zoomed.toggled_at(&VIEWPORT, MIDDLE).is_fit());
    }

    #[test]
    fn actual_size_keeps_the_middle_of_the_viewport() {
        let zoomed = View::fit().zoomed_around(&VIEWPORT, [100.0, 200.0], 2.0);
        let middle_before = zoomed.photo_position_at(&VIEWPORT, MIDDLE);

        let actual = zoomed.at_actual_size(&VIEWPORT);

        assert_eq!(actual.scale(&VIEWPORT), ACTUAL_SIZE);
        assert_close(actual.photo_position_at(&VIEWPORT, MIDDLE), middle_before);
    }

    #[test]
    fn panning_moves_the_photo_with_the_drag() {
        let actual = View::fit().at_actual_size(&VIEWPORT);
        let grabbed = actual.photo_position_at(&VIEWPORT, MIDDLE);

        let panned = actual.panned_by(&VIEWPORT, [-50.0, 30.0]);

        assert_close(panned.photo_position_at(&VIEWPORT, [350.0, 330.0]), grabbed);
    }

    #[test]
    fn panning_cannot_open_a_margin_beside_an_overflowing_photo() {
        let actual = View::fit().at_actual_size(&VIEWPORT);

        let top_left = actual.panned_by(&VIEWPORT, [1e6, 1e6]).placement(&VIEWPORT);
        let bottom_right = actual
            .panned_by(&VIEWPORT, [-1e6, -1e6])
            .placement(&VIEWPORT);

        assert_eq!(top_left.photo_min, [0.0, 0.0]);
        assert_eq!(bottom_right.photo_min, [3200.0, 1400.0]);
        assert_eq!(top_left.screen_size, VIEWPORT.size);
    }

    #[test]
    fn dimension_smaller_than_the_viewport_stays_centered() {
        // 25 %: 1000×500 on screen, wider than the viewport but not taller.
        let zoomed = View::fit().zoomed_around(&VIEWPORT, MIDDLE, 1.25);

        let placement = zoomed
            .panned_by(&VIEWPORT, [0.0, 500.0])
            .placement(&VIEWPORT);

        assert_eq!(placement.screen_min[1], 50.0);
        assert_eq!(placement.screen_size, [800.0, 500.0]);
        assert_eq!(placement.photo_size, [3200.0, 2000.0]);
    }

    #[test]
    fn photo_smaller_than_the_viewport_is_already_at_actual_size() {
        let small = Viewport {
            photo: [100, 50],
            size: [800.0, 600.0],
        };

        assert!(View::fit().toggled_at(&small, MIDDLE).is_fit());
        assert_eq!(View::fit().placement(&small).screen_size, [100.0, 50.0]);
    }
}
