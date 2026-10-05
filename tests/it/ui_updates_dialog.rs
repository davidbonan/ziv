use egui::accesskit::Role;
use egui::vec2;
use egui_commonmark::CommonMarkCache;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::update::domain::published_release::PublishedRelease;
use ziv::update::domain::update_state::UpdateState;
use ziv::update::domain::version::Version;
use ziv::update::ui::running_version_button::running_version_button;
use ziv::update::ui::update_status::{CHECKING_LABEL, UP_TO_DATE_LABEL};
use ziv::update::ui::update_strip::INSTALL_LABEL;
use ziv::update::ui::updates_dialog::{
    CHECK_LABEL, CLOSE_LABEL, UNBUNDLED_LABEL, UpdatesDialogIntent, UpdatesShown, updates_dialog,
};

use crate::themed::is_themed;

/// A spinner repaints for ever: frames are counted, the theme taking the first.
const THEMED_STEPS: usize = 3;

#[derive(Default)]
struct Dialog {
    notes_layout: CommonMarkCache,
    asked: Option<UpdatesDialogIntent>,
}

fn version(text: &str) -> Version {
    Version::parse(text).expect("a version")
}

fn available() -> UpdateState {
    UpdateState::Available(PublishedRelease {
        version: version("0.2.0"),
        asset_url: "https://example.invalid/ziv-macos.zip".to_owned(),
    })
}

/// The dialog of a bundled app in `state`, of an unbundled one without it.
fn dialog_harness(state: Option<UpdateState>, can_install: bool) -> Harness<'static, Dialog> {
    let mut harness = Harness::builder()
        .with_size(vec2(520.0, 520.0))
        .build_ui_state(
            move |ui, dialog: &mut Dialog| {
                if !is_themed(ui) {
                    return;
                }
                let shown = UpdatesShown {
                    running: version("0.1.0"),
                    state: state.as_ref(),
                    can_install,
                };
                let asked = updates_dialog(ui, &shown, &mut dialog.notes_layout);
                dialog.asked = asked.or(dialog.asked);
            },
            Dialog::default(),
        );
    harness.run_steps(THEMED_STEPS);
    harness
}

fn in_state(state: UpdateState) -> Harness<'static, Dialog> {
    dialog_harness(Some(state), true)
}

fn asked_by_clicking(state: UpdateState, label: &str) -> Option<UpdatesDialogIntent> {
    let mut harness = in_state(state);
    harness.get_by_role_and_label(Role::Button, label).click();
    harness.run_steps(THEMED_STEPS);
    harness.state().asked
}

fn is_disabled(harness: &Harness<'static, Dialog>, label: &str) -> bool {
    let button = harness.get_by_role_and_label(Role::Button, label);
    button.accesskit_node().is_disabled()
}

#[test]
fn the_dialog_shows_the_running_version_and_the_release_notes() {
    let harness = in_state(UpdateState::Idle);

    assert!(harness.query_by_label("ziv 0.1.0").is_some());
    assert!(harness.query_by_label_contains("First release").is_some());
}

#[test]
fn each_state_of_the_update_is_said() {
    let said = [
        (UpdateState::Checking, CHECKING_LABEL),
        (UpdateState::UpToDate, UP_TO_DATE_LABEL),
        (available(), "Version 0.2.0 is available"),
        (
            UpdateState::Downloading(version("0.2.0")),
            "Downloading 0.2.0…",
        ),
        (UpdateState::Failed("offline".to_owned()), "offline"),
    ];
    for (state, label) in said {
        assert!(in_state(state).query_by_label(label).is_some(), "{label}");
    }
}

#[test]
fn each_button_asks_what_it_says() {
    assert_eq!(
        asked_by_clicking(UpdateState::Idle, CHECK_LABEL),
        Some(UpdatesDialogIntent::Check)
    );
    assert_eq!(
        asked_by_clicking(available(), INSTALL_LABEL),
        Some(UpdatesDialogIntent::Install)
    );
    assert_eq!(
        asked_by_clicking(UpdateState::Idle, CLOSE_LABEL),
        Some(UpdatesDialogIntent::Close)
    );
}

#[test]
fn nothing_can_be_started_while_an_operation_runs() {
    assert!(is_disabled(&in_state(UpdateState::Checking), CHECK_LABEL));
    let downloading = in_state(UpdateState::Downloading(version("0.2.0")));
    assert!(is_disabled(&downloading, CHECK_LABEL));
    assert!(downloading.query_by_label(INSTALL_LABEL).is_none());
}

#[test]
fn an_install_waits_for_what_a_relaunch_would_cut_short() {
    let harness = dialog_harness(Some(available()), false);

    assert!(is_disabled(&harness, INSTALL_LABEL));
}

#[test]
fn an_unbundled_app_says_it_cannot_update_itself() {
    let harness = dialog_harness(None, true);

    assert!(harness.query_by_label(UNBUNDLED_LABEL).is_some());
    assert!(harness.query_by_label(CHECK_LABEL).is_none());
}

#[test]
fn the_running_version_is_a_button() {
    let mut harness = Harness::builder().build_ui_state(
        |ui, is_clicked: &mut bool| {
            if is_themed(ui) {
                *is_clicked |= running_version_button(ui, version("0.1.0")).clicked();
            }
        },
        false,
    );
    harness.run();

    harness
        .get_by_role_and_label(Role::Button, "ziv 0.1.0")
        .click();
    harness.run();

    assert!(*harness.state());
}
