use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::develop::domain::zone::PersonPart;
use ziv::zones::domain::people_pick::PeoplePick;
use ziv::zones::domain::photo_view::PhotoRegion;
use ziv::zones::ui::people_picker::{
    CANCEL_LABEL, CREATE_LABEL, PeoplePickerIntent, people_picker, person_label,
};

use crate::themed::is_themed;

struct Picker {
    pick: PeoplePick,
    intent: Option<PeoplePickerIntent>,
}

fn two_persons() -> PeoplePick {
    let person_at = |left: f32| PhotoRegion {
        min: [left, 0.2],
        size: [0.2, 0.6],
    };
    PeoplePick::among(vec![person_at(0.1), person_at(0.6)])
}

fn picker(pick: PeoplePick) -> Harness<'static, Picker> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(640.0, 400.0))
        .build_ui_state(
            |ui, picker: &mut Picker| {
                if is_themed(ui) {
                    let output = people_picker(ui, ui.max_rect(), &picker.pick);
                    picker.pick = output.pick;
                    picker.intent = output.intent.or(picker.intent);
                }
            },
            Picker { pick, intent: None },
        );
    harness.run();
    harness
}

fn click(harness: &mut Harness<'static, Picker>, label: &str) {
    harness.get_by_role_and_label(Role::Button, label).click();
    harness.run();
}

fn is_create_disabled(harness: &Harness<'static, Picker>) -> bool {
    harness
        .get_by_role_and_label(Role::Button, CREATE_LABEL)
        .accesskit_node()
        .is_disabled()
}

#[test]
fn every_person_is_listed_and_masks_wait_for_a_part() {
    let harness = picker(two_persons());

    for person in 0..2 {
        assert!(harness.query_by_label(&person_label(person)).is_some());
    }
    assert!(is_create_disabled(&harness));
}

#[test]
fn chosen_parts_of_the_chosen_persons_are_asked_for() {
    let mut harness = picker(two_persons());

    click(&mut harness, PersonPart::Hair.name());
    click(&mut harness, PersonPart::Skin.name());
    click(&mut harness, &person_label(0));
    click(&mut harness, CREATE_LABEL);

    let Picker { pick, intent } = harness.state();
    assert_eq!(*intent, Some(PeoplePickerIntent::Create));
    assert_eq!(pick.chosen_parts(), [PersonPart::Skin, PersonPart::Hair]);
    assert_eq!(pick.chosen_persons().len(), 1);
}

#[test]
fn masks_cannot_be_created_for_nobody() {
    let mut harness = picker(two_persons().with_part_toggled(PersonPart::Eyes));

    click(&mut harness, &person_label(0));
    click(&mut harness, &person_label(1));

    assert!(is_create_disabled(&harness));
}

#[test]
fn cancel_and_escape_leave_without_masks() {
    let mut cancelled = picker(two_persons());
    click(&mut cancelled, CANCEL_LABEL);

    let mut escaped = picker(two_persons());
    escaped.key_press(egui::Key::Escape);
    escaped.run();

    for harness in [cancelled, escaped] {
        assert_eq!(harness.state().intent, Some(PeoplePickerIntent::Cancel));
    }
}
