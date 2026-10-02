use crate::themed::is_themed;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::library::ui::empty_state::{EMPTY_LIBRARY_LABEL, OPEN_BUTTON_LABEL, empty_state};

fn empty_state_harness() -> Harness<'static, bool> {
    let mut harness = Harness::new_ui_state(
        |ui, open_requested: &mut bool| {
            if is_themed(ui) {
                *open_requested |= empty_state(ui);
            }
        },
        false,
    );
    harness.run();
    harness
}

#[test]
fn empty_library_tells_the_user_there_is_no_photo() {
    let harness = empty_state_harness();

    assert!(harness.query_by_label(EMPTY_LIBRARY_LABEL).is_some());
    assert!(!harness.state());
}

#[test]
fn open_button_asks_to_open_photos() {
    let mut harness = empty_state_harness();

    harness.get_by_label(OPEN_BUTTON_LABEL).click();
    harness.run();

    assert!(harness.state());
}
