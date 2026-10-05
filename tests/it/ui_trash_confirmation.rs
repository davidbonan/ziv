use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::library::ui::trash_confirmation::{
    CANCEL_TRASH_LABEL, CONFIRM_TRASH_LABEL, TrashConfirmationIntent, trash_confirmation,
};

use crate::themed::is_themed;

fn confirmation_harness(photo_count: usize) -> Harness<'static, Option<TrashConfirmationIntent>> {
    let mut harness = Harness::builder()
        .with_size(vec2(420.0, 200.0))
        .build_ui_state(
            move |ui, answer: &mut Option<TrashConfirmationIntent>| {
                if !is_themed(ui) {
                    return;
                }
                if let Some(intent) = trash_confirmation(ui, photo_count) {
                    *answer = Some(intent);
                }
            },
            None,
        );
    harness.run();
    harness
}

fn answered_by_clicking(label: &str) -> Option<TrashConfirmationIntent> {
    let mut harness = confirmation_harness(3);
    harness.get_by_label(label).click();
    harness.run();
    *harness.state()
}

#[test]
fn confirmation_says_how_many_photos_go_to_the_trash() {
    assert!(
        confirmation_harness(3)
            .query_by_label("Move 3 photos to the Trash?")
            .is_some()
    );
    assert!(
        confirmation_harness(1)
            .query_by_label("Move 1 photo to the Trash?")
            .is_some()
    );
}

#[test]
fn each_button_answers_what_it_says() {
    assert_eq!(
        answered_by_clicking(CONFIRM_TRASH_LABEL),
        Some(TrashConfirmationIntent::Confirm)
    );
    assert_eq!(
        answered_by_clicking(CANCEL_TRASH_LABEL),
        Some(TrashConfirmationIntent::Cancel)
    );
}
