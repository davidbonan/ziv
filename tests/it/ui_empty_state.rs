use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::library::ui::empty_state::{EMPTY_LIBRARY_LABEL, empty_state};

#[test]
fn empty_library_tells_the_user_there_is_no_photo() {
    let mut harness = Harness::new_ui(empty_state);
    harness.run();

    assert!(harness.query_by_label(EMPTY_LIBRARY_LABEL).is_some());
}
