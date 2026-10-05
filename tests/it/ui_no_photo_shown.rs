use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::library::domain::series_filter::SeriesFilter;
use ziv::library::ui::no_photo_shown::{SHOW_ALL_LABEL, no_photo_shown};

use crate::themed::is_themed;

#[test]
fn an_empty_result_names_the_filter_and_offers_to_show_every_photo() {
    let mut harness = Harness::builder()
        .with_size(vec2(400.0, 300.0))
        .build_ui_state(
            |ui, is_asked: &mut bool| {
                if is_themed(ui) {
                    *is_asked |= no_photo_shown(ui, SeriesFilter::at_least(3));
                }
            },
            false,
        );
    harness.run();
    assert!(
        harness
            .query_by_label("No photo at 3 stars or more")
            .is_some()
    );

    harness.get_by_label(SHOW_ALL_LABEL).click();
    harness.run();

    assert!(*harness.state());
}
