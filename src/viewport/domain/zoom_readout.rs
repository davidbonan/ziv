use super::view::View;

/// The zoom as the top bar says it: "Fit · 24 %" at fit, "150 %" otherwise.
/// `scale` is screen pixels per photo pixel.
pub fn zoom_readout(view: &View, scale: f32) -> String {
    let percent = (scale * 100.0).round();
    match view.is_fit() {
        true => format!("Fit · {percent} %"),
        false => format!("{percent} %"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewport::domain::view::Viewport;

    const VIEWPORT: Viewport = Viewport {
        photo: [4000, 2000],
        size: [1000.0, 800.0],
    };

    #[test]
    fn view_at_fit_says_fit_and_its_scale() {
        let view = View::fit();

        assert_eq!(zoom_readout(&view, view.scale(&VIEWPORT)), "Fit · 25 %");
    }

    #[test]
    fn zoomed_view_says_its_scale_only() {
        let view = View::fit().at_actual_size(&VIEWPORT);

        assert_eq!(zoom_readout(&view, view.scale(&VIEWPORT)), "100 %");
    }
}
