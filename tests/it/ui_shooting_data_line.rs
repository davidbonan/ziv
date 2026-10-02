use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::photo::domain::shooting_data::ShootingData;
use ziv::photo::ui::shooting_data_line::shooting_data_line;

use crate::themed::is_themed;

const PANEL_WIDTH: f32 = 268.0;
const AFTER_LABEL: &str = "after the line";

const SHOT: ShootingData = ShootingData {
    iso: Some(400),
    focal_length: Some(35.0),
    aperture: Some(2.8),
    shutter_speed: Some(0.004),
};

fn line_harness(shooting_data: ShootingData) -> Harness<'static> {
    let mut harness = Harness::builder()
        .with_size(vec2(PANEL_WIDTH, 200.0))
        .build_ui(move |ui| {
            if is_themed(ui) {
                shooting_data_line(ui, &shooting_data);
                ui.label(AFTER_LABEL);
            }
        });
    harness.run();
    harness
}

#[test]
fn the_line_spreads_the_shooting_data_over_the_width_in_order() {
    let harness = line_harness(SHOT);

    let edges = ["ISO 400", "35 mm", "f/2.8", "1/250 s"].map(|label| {
        let place = harness.get_by_label(label).rect();
        (place.left(), place.right())
    });

    assert!(edges.is_sorted_by(|before, after| before.1 < after.0));
    assert!(edges[3].1 > PANEL_WIDTH - 12.0, "{edges:?}");
}

#[test]
fn the_place_of_the_line_is_kept_without_shooting_data() {
    let with_data = line_harness(SHOT);
    let without = line_harness(ShootingData::default());

    let top_after = |harness: &Harness<'_>| harness.get_by_label(AFTER_LABEL).rect().top();
    assert_eq!(top_after(&without), top_after(&with_data));
}
