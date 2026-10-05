use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::update::domain::published_release::PublishedRelease;
use ziv::update::domain::update_state::UpdateState;
use ziv::update::domain::version::Version;
use ziv::update::ui::update_strip::{
    INSTALL_LABEL, LATER_LABEL, UpdateStripIntent, UpdateStripShown, update_strip,
};

use crate::themed::is_themed;

/// A spinner repaints for ever: frames are counted, the theme taking the first.
const THEMED_STEPS: usize = 3;

fn version() -> Version {
    Version::parse("0.2.0").expect("a version")
}

fn available() -> UpdateState {
    UpdateState::Available(PublishedRelease {
        version: version(),
        asset_url: "https://example.invalid/ziv-macos.zip".to_owned(),
    })
}

fn strip_harness(state: UpdateState) -> Harness<'static, Option<UpdateStripIntent>> {
    let mut harness = Harness::builder()
        .with_size(vec2(640.0, 200.0))
        .build_ui_state(
            move |ui, asked: &mut Option<UpdateStripIntent>| {
                if !is_themed(ui) {
                    return;
                }
                let shown = UpdateStripShown {
                    state: &state,
                    can_install: true,
                };
                *asked = update_strip(ui, ui.max_rect(), &shown).or(*asked);
            },
            None,
        );
    harness.run_steps(THEMED_STEPS);
    harness
}

fn asked_by_clicking(label: &str) -> Option<UpdateStripIntent> {
    let mut harness = strip_harness(available());
    harness.get_by_role_and_label(Role::Button, label).click();
    harness.run_steps(THEMED_STEPS);
    *harness.state()
}

#[test]
fn an_available_update_is_offered_with_its_version() {
    let harness = strip_harness(available());

    assert!(
        harness
            .query_by_label("Version 0.2.0 is available")
            .is_some()
    );
    assert_eq!(
        asked_by_clicking(INSTALL_LABEL),
        Some(UpdateStripIntent::Install)
    );
    assert_eq!(
        asked_by_clicking(LATER_LABEL),
        Some(UpdateStripIntent::Later)
    );
}

#[test]
fn an_install_shows_its_progress_without_buttons() {
    let downloading = strip_harness(UpdateState::Downloading(version()));
    assert!(downloading.query_by_label("Downloading 0.2.0…").is_some());
    assert!(downloading.query_by_label(INSTALL_LABEL).is_none());

    let installing = strip_harness(UpdateState::Installing(version()));
    assert!(installing.query_by_label("Installing 0.2.0…").is_some());
    assert!(installing.query_by_label(LATER_LABEL).is_none());
}

#[test]
fn nothing_is_shown_without_an_update() {
    for state in [
        UpdateState::Idle,
        UpdateState::Checking,
        UpdateState::UpToDate,
    ] {
        let harness = strip_harness(state);

        assert!(harness.query_by_label(INSTALL_LABEL).is_none());
    }
}
