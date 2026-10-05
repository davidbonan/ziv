use egui::accesskit::Role;
use egui::vec2;
use egui_commonmark::CommonMarkCache;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::update::ui::updates_dialog::CLOSE_LABEL;
use ziv::update::ui::whats_new::{WHATS_NEW_TITLE, whats_new};

use crate::themed::is_themed;

#[test]
fn whats_new_shows_the_release_notes_until_closed() {
    let mut harness = Harness::builder()
        .with_size(vec2(520.0, 460.0))
        .build_ui_state(
            |ui, (notes_layout, is_closed): &mut (CommonMarkCache, bool)| {
                if is_themed(ui) {
                    *is_closed |= whats_new(ui, notes_layout);
                }
            },
            (CommonMarkCache::default(), false),
        );
    harness.run();
    assert!(harness.query_by_label(WHATS_NEW_TITLE).is_some());
    assert!(harness.query_by_label_contains("First release").is_some());

    harness
        .get_by_role_and_label(Role::Button, CLOSE_LABEL)
        .click();
    harness.run();

    assert!(harness.state().1);
}
