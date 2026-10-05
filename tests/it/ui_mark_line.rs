use egui::accesskit::Toggled;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::library::domain::mark::{Mark, Rating};
use ziv::library::ui::mark_line::{REJECTED_LABEL, mark_line};

use crate::themed::is_themed;

struct Line {
    mark: Mark,
    asked: Option<Rating>,
}

fn line_harness(mark: Mark) -> Harness<'static, Line> {
    let line = Line { mark, asked: None };
    let mut harness = Harness::builder()
        .with_size(vec2(300.0, 40.0))
        .build_ui_state(
            |ui, line: &mut Line| {
                if !is_themed(ui) {
                    return;
                }
                if let Some(rating) = mark_line(ui, line.mark) {
                    line.asked = Some(rating);
                }
            },
            line,
        );
    harness.run();
    harness
}

fn three_stars(is_rejected: bool) -> Mark {
    Mark {
        rating: Rating::of(3),
        is_rejected,
    }
}

#[test]
fn stars_are_lit_up_to_the_rating() {
    let harness = line_harness(three_stars(false));

    let is_lit = |label| harness.get_by_label(label).accesskit_node().toggled();
    assert_eq!(is_lit("3 stars"), Some(Toggled::True));
    assert_eq!(is_lit("4 stars"), Some(Toggled::False));
    assert!(harness.query_by_label(REJECTED_LABEL).is_none());
}

#[test]
fn clicking_a_star_asks_for_its_rating() {
    let mut harness = line_harness(three_stars(false));

    harness.get_by_label("5 stars").click();
    harness.run();

    assert_eq!(harness.state().asked, Some(Rating::of(5)));
}

#[test]
fn a_rejected_photo_says_so_and_keeps_its_stars() {
    let harness = line_harness(three_stars(true));

    assert!(harness.query_by_label(REJECTED_LABEL).is_some());
    let third = harness.get_by_label("3 stars");
    assert_eq!(third.accesskit_node().toggled(), Some(Toggled::True));
}
