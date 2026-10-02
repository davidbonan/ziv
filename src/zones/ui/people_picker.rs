use egui::{Align, Align2, Button, Layout, Rect, RichText, Stroke, StrokeKind, vec2};

use crate::design::ui::floating_pill::floating_pill;
use crate::design::ui::theme::{CONTROL_RADIUS, color, medium, regular, space, type_size};
use crate::develop::domain::zone::PersonPart;
use crate::zones::domain::people_pick::PeoplePick;

pub const PICKER_TITLE: &str = "People";
pub const CREATE_LABEL: &str = "Create masks";
pub const CANCEL_LABEL: &str = "Cancel";

const ROW_NAME_WIDTH: f32 = 64.0;
const CHOSEN_OUTLINE_WIDTH: f32 = 2.0;
const NUMBER_BADGE_SIZE: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeoplePickerIntent {
    Create,
    Cancel,
}

pub struct PeoplePickerOutput {
    pub pick: PeoplePick,
    pub intent: Option<PeoplePickerIntent>,
}

pub fn person_label(index: usize) -> String {
    format!("Person {}", index + 1)
}

fn row_name(ui: &mut egui::Ui, name: &str) {
    let size = vec2(ROW_NAME_WIDTH, ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        ui.set_min_size(size);
        let name = RichText::new(name)
            .font(regular(type_size::BODY))
            .color(color::TEXT_MUTED);
        ui.label(name);
    });
}

fn persons_row(ui: &mut egui::Ui, pick: PeoplePick) -> PeoplePick {
    let chosen: Vec<bool> = pick.persons().map(|(_, is_chosen)| is_chosen).collect();
    let mut toggled = None;
    ui.horizontal_wrapped(|ui| {
        row_name(ui, "Persons");
        for (index, is_chosen) in chosen.into_iter().enumerate() {
            let person = Button::new(person_label(index)).selected(is_chosen);
            if ui.add(person).clicked() {
                toggled = Some(index);
            }
        }
    });
    match toggled {
        Some(person) => pick.with_person_toggled(person),
        None => pick,
    }
}

fn parts_row(ui: &mut egui::Ui, pick: PeoplePick) -> PeoplePick {
    let mut toggled = None;
    ui.horizontal_wrapped(|ui| {
        row_name(ui, "Parts");
        for part in PersonPart::ALL {
            let button = Button::new(part.name()).selected(pick.is_part_chosen(part));
            if ui.add(button).clicked() {
                toggled = Some(part);
            }
        }
    });
    match toggled {
        Some(part) => pick.with_part_toggled(part),
        None => pick,
    }
}

fn footer(ui: &mut egui::Ui, can_create: bool) -> Option<PeoplePickerIntent> {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        let create =
            Button::new(RichText::new(CREATE_LABEL).color(color::HAIRLINE)).fill(color::ACCENT);
        if ui.add_enabled(can_create, create).clicked() {
            return Some(PeoplePickerIntent::Create);
        }
        ui.button(CANCEL_LABEL)
            .clicked()
            .then_some(PeoplePickerIntent::Cancel)
    })
    .inner
}

fn key_intent(ui: &egui::Ui, can_create: bool) -> Option<PeoplePickerIntent> {
    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        return Some(PeoplePickerIntent::Cancel);
    }
    let is_confirmed = can_create && ui.input(|input| input.key_pressed(egui::Key::Enter));
    is_confirmed.then_some(PeoplePickerIntent::Create)
}

/// Who to mask and which parts, floating over the bottom of `over`. Returns
/// the pick as the user left it this frame and what they asked for.
pub fn people_picker(ui: &egui::Ui, over: Rect, pick: &PeoplePick) -> PeoplePickerOutput {
    let pick = pick.clone();
    floating_pill(ui, ("people picker", over), |ui| {
        ui.spacing_mut().item_spacing.y = space::S;
        let title = RichText::new(PICKER_TITLE)
            .font(medium(type_size::TITLE))
            .color(color::TEXT);
        ui.label(title);
        let pick = persons_row(ui, pick);
        let pick = parts_row(ui, pick);
        ui.add_space(space::XS);
        let can_create = pick.can_create_masks();
        let intent = footer(ui, can_create).or(key_intent(ui, can_create));
        PeoplePickerOutput { pick, intent }
    })
}

fn paint_number(painter: &egui::Painter, outline: Rect, (index, is_chosen): (usize, bool)) {
    let badge = Rect::from_min_size(outline.min, vec2(NUMBER_BADGE_SIZE, NUMBER_BADGE_SIZE));
    let (fill, ink) = match is_chosen {
        true => (color::ACCENT, color::HAIRLINE),
        false => (color::RAISED, color::TEXT_MUTED),
    };
    painter.rect_filled(badge, CONTROL_RADIUS, fill);
    painter.text(
        badge.center(),
        Align2::CENTER_CENTER,
        index + 1,
        medium(type_size::CAPTION),
        ink,
    );
}

/// Outlines each person on the photo with their number; `whole_photo` is
/// where the whole photo is on screen, `area` what is seen of it.
pub fn person_outlines(ui: &egui::Ui, (area, whole_photo): (Rect, Rect), pick: &PeoplePick) {
    let painter = ui.painter_at(area);
    for (index, (person, is_chosen)) in pick.persons().enumerate() {
        let min = whole_photo.min + vec2(person.min[0], person.min[1]) * whole_photo.size();
        let size = vec2(person.size[0], person.size[1]) * whole_photo.size();
        let outline = Rect::from_min_size(min, size);
        let stroke = match is_chosen {
            true => Stroke::new(CHOSEN_OUTLINE_WIDTH, color::ACCENT),
            false => Stroke::new(1.0, color::TEXT_MUTED),
        };
        painter.rect_stroke(outline, CONTROL_RADIUS, stroke, StrokeKind::Outside);
        paint_number(&painter, outline, (index, is_chosen));
    }
}
