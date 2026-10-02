use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::histogram::domain::histogram::Histogram;
use ziv::histogram::ui::histogram_plot::{HISTOGRAM_HEIGHT, HISTOGRAM_LABEL, histogram_plot};

const PANEL_WIDTH: f32 = 268.0;
/// What the harness keeps free on each side of its content.
const HARNESS_MARGIN: f32 = 8.0;
const PLOT_SIZE: egui::Vec2 = vec2(PANEL_WIDTH - 2.0 * HARNESS_MARGIN, HISTOGRAM_HEIGHT);

fn plot_harness(histogram: Option<Histogram>) -> Harness<'static> {
    let mut harness = Harness::builder()
        .with_size(vec2(PANEL_WIDTH, 200.0))
        .build_ui(move |ui| histogram_plot(ui, histogram.as_ref()));
    harness.run();
    harness
}

fn plot_size(harness: &Harness<'_>) -> egui::Vec2 {
    harness.get_by_label(HISTOGRAM_LABEL).rect().size()
}

#[test]
fn the_histogram_is_found_by_its_name_across_the_width() {
    let grey_and_white = [[100, 100, 100, 255], [255, 255, 255, 255]];
    let histogram = Histogram::of_display_pixels(grey_and_white.as_flattened());

    let harness = plot_harness(Some(histogram));

    assert_eq!(plot_size(&harness), PLOT_SIZE);
}

#[test]
fn the_place_of_the_histogram_is_kept_without_a_photo() {
    let harness = plot_harness(None);

    assert_eq!(plot_size(&harness), PLOT_SIZE);
}
