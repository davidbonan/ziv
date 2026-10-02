use std::path::Path;

use egui::{
    Align, Align2, Color32, Key, Layout, Rect, RichText, Sense, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::design::ui::icon_button::{Icon, icon_button};
use crate::design::ui::theme::{CONTROL_RADIUS, color, medium, regular, space, type_size};
use crate::library::domain::import_day::ImportDay;
use crate::library::domain::series::COVER_PHOTO_COUNT;

pub const SERIES_SIDEBAR_WIDTH: f32 = 232.0;
pub const SERIES_TITLE: &str = "Series";
pub const IMPORT_BUTTON_LABEL: &str = "Import…";
pub const MISSING_SERIES_CAPTION: &str = "Folder not found";
pub const RENAME_LABEL: &str = "Rename";
pub const SHOW_IN_FINDER_LABEL: &str = "Show in Finder";
pub const LOCATE_LABEL: &str = "Locate…";
pub const REMOVE_LABEL: &str = "Remove";
pub const SERIES_NAME_FIELD_LABEL: &str = "Series name";

const ROW_HEIGHT: f32 = 48.0;
const LINE_OFFSET: f32 = 9.0;
const COVER_SIDE: f32 = 40.0;
const COVER_GAP: f32 = 1.0;
const MISSING_COVER_TINT: Color32 = Color32::from_gray(70);

/// The thumbnails of the first photos of a series; `None` for one not there yet.
pub type SeriesCover<'a> = [Option<&'a egui::TextureHandle>; COVER_PHOTO_COUNT];

pub struct SeriesRow<'a> {
    pub name: &'a str,
    pub photo_count: usize,
    pub edited_count: usize,
    pub imported_on: ImportDay,
    pub cover: SeriesCover<'a>,
    /// None of its photos is where the series has it.
    pub is_missing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarIntent {
    Open(usize),
    Import,
    Rename { index: usize, name: String },
    ShowInFinder(usize),
    Locate(usize),
    Remove(usize),
}

/// The series whose name is being typed, and what was typed so far.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Renaming {
    index: usize,
    draft: String,
    is_field_focused: bool,
}

/// What is said of a series under its name.
fn caption(row: &SeriesRow<'_>) -> String {
    if row.is_missing {
        return MISSING_SERIES_CAPTION.to_owned();
    }
    let SeriesRow {
        photo_count,
        edited_count,
        imported_on,
        ..
    } = row;
    format!("{edited_count} of {photo_count} edited, {imported_on}")
}

/// What assistive technology and tests read for a series.
pub fn series_label(row: &SeriesRow<'_>) -> String {
    format!("{}, {}", row.name, caption(row))
}

/// The middle square of a picture of that aspect ratio, in texture coordinates.
fn middle_square(aspect: f32) -> Rect {
    let (width, height) = match aspect > 1.0 {
        true => (1.0 / aspect, 1.0),
        false => (1.0, aspect),
    };
    Rect::from_center_size(pos2(0.5, 0.5), vec2(width, height))
}

fn paint_cover(painter: &egui::Painter, area: Rect, row: &SeriesRow<'_>) {
    let cell_side = (COVER_SIDE - COVER_GAP) / 2.0;
    let tint = match row.is_missing {
        true => MISSING_COVER_TINT,
        false => Color32::WHITE,
    };
    for (index, photo) in row.cover.iter().enumerate() {
        let column = (index % 2) as f32;
        let line = (index / 2) as f32;
        let corner = area.min + vec2(column, line) * (cell_side + COVER_GAP);
        let cell = Rect::from_min_size(corner, vec2(cell_side, cell_side));
        match photo {
            Some(texture) => {
                let shown = middle_square(texture.aspect_ratio());
                painter.image(texture.id(), cell, shown, tint);
            }
            None => {
                painter.rect_filled(cell, 0.0, color::CANVAS);
            }
        }
    }
}

struct RowPlaces {
    cover: Rect,
    name: Rect,
    caption: egui::Pos2,
}

fn row_places(area: Rect) -> RowPlaces {
    let cover = Rect::from_center_size(
        area.left_center() + vec2(space::XS + COVER_SIDE / 2.0, 0.0),
        vec2(COVER_SIDE, COVER_SIDE),
    );
    let left = cover.right() + space::S;
    let name_line = area.center().y - LINE_OFFSET;
    let name = Rect::from_min_max(
        pos2(left, name_line - LINE_OFFSET),
        pos2(area.right() - space::XS, name_line + LINE_OFFSET),
    );
    RowPlaces {
        cover,
        name,
        caption: pos2(left, area.center().y + LINE_OFFSET),
    }
}

fn row_menu(ui: &mut egui::Ui, index: usize, row: &SeriesRow<'_>) -> Option<SidebarIntent> {
    if ui.button(RENAME_LABEL).clicked() {
        let renaming = Renaming {
            index,
            draft: row.name.to_owned(),
            is_field_focused: false,
        };
        ui.data_mut(|data| data.insert_temp(renaming_id(), Some(renaming)));
    }
    let finder = egui::Button::new(SHOW_IN_FINDER_LABEL);
    if ui.add_enabled(!row.is_missing, finder).clicked() {
        return Some(SidebarIntent::ShowInFinder(index));
    }
    if row.is_missing && ui.button(LOCATE_LABEL).clicked() {
        return Some(SidebarIntent::Locate(index));
    }
    ui.button(REMOVE_LABEL)
        .clicked()
        .then_some(SidebarIntent::Remove(index))
}

fn renaming_id() -> egui::Id {
    egui::Id::new("series being renamed")
}

/// The name field of the row being renamed. Returns the name once confirmed.
fn name_field(ui: &mut egui::Ui, place: Rect, mut renaming: Renaming) -> Option<SidebarIntent> {
    let field = egui::TextEdit::singleline(&mut renaming.draft).font(medium(type_size::BODY));
    let response = ui.put(place, field);
    response
        .widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, SERIES_NAME_FIELD_LABEL));
    if !renaming.is_field_focused {
        response.request_focus();
        renaming.is_field_focused = true;
    }
    let is_over = response.lost_focus();
    let is_confirmed = is_over && ui.input(|input| input.key_pressed(Key::Enter));
    let left = (!is_over).then(|| renaming.clone());
    ui.data_mut(|data| data.insert_temp(renaming_id(), left));
    is_confirmed.then_some(SidebarIntent::Rename {
        index: renaming.index,
        name: renaming.draft,
    })
}

fn series_row(
    ui: &mut egui::Ui,
    (index, row): (usize, &SeriesRow<'_>),
    is_open: bool,
) -> Option<SidebarIntent> {
    let size = vec2(ui.available_width(), ROW_HEIGHT);
    let (area, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            true,
            is_open,
            series_label(row),
        )
    });
    if is_open || response.hovered() {
        ui.painter()
            .rect_filled(area, CONTROL_RADIUS, color::RAISED);
    }
    let places = row_places(area);
    paint_cover(ui.painter(), places.cover, row);
    let text = ui.painter().with_clip_rect(area.shrink(space::XS));
    let (name_ink, caption_ink) = match (row.is_missing, is_open) {
        (true, _) => (color::TEXT_DISABLED, color::DANGER),
        (false, true) => (color::ACCENT, color::TEXT_MUTED),
        (false, false) => (color::TEXT, color::TEXT_MUTED),
    };
    text.text(
        places.caption,
        Align2::LEFT_CENTER,
        caption(row),
        regular(type_size::CAPTION),
        caption_ink,
    );
    let renaming: Option<Renaming> = ui.data(|data| data.get_temp(renaming_id())).flatten();
    if let Some(renaming) = renaming.filter(|renaming| renaming.index == index) {
        return name_field(ui, places.name, renaming);
    }
    text.text(
        places.name.left_center(),
        Align2::LEFT_CENTER,
        row.name,
        medium(type_size::BODY),
        name_ink,
    );
    let mut asked = None;
    response.context_menu(|ui| asked = row_menu(ui, index, row));
    asked.or(response.clicked().then_some(SidebarIntent::Open(index)))
}

pub struct SeriesShown<'a> {
    pub rows: &'a [SeriesRow<'a>],
    pub open: Option<usize>,
    /// Why the catalog is not kept, when it is not.
    pub warning: Option<&'a str>,
}

/// Why the series of this run will be forgotten: what is stored cannot be used.
pub fn unusable_catalog_warning(file: Option<&Path>, reason: &str) -> String {
    let file = file.map_or("The catalog".to_owned(), |file| file.display().to_string());
    format!("{file} cannot be used: {reason}. Series imported now will not be remembered.")
}

fn catalog_warning(ui: &mut egui::Ui, warning: &str) {
    egui::Panel::bottom("catalog warning")
        .frame(egui::Frame::new().inner_margin(space::S))
        .show(ui, |ui| {
            let text = RichText::new(warning)
                .font(regular(type_size::CAPTION))
                .color(color::DANGER);
            ui.label(text);
        });
}

/// The list of the series. Returns what the user asked for this frame, if anything.
pub fn series_sidebar(ui: &mut egui::Ui, shown: &SeriesShown<'_>) -> Option<SidebarIntent> {
    let SeriesShown {
        rows,
        open,
        warning,
    } = shown;
    if let Some(warning) = warning {
        catalog_warning(ui, warning);
    }
    let mut intent = None;
    ui.horizontal(|ui| {
        ui.add_space(space::S);
        ui.label(RichText::new(SERIES_TITLE).font(medium(type_size::BODY)));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, Icon::Add, IMPORT_BUTTON_LABEL).clicked() {
                intent = Some(SidebarIntent::Import);
            }
        });
    });
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = space::XS;
        for (index, row) in rows.iter().enumerate() {
            let asked = series_row(ui, (index, row), *open == Some(index));
            intent = asked.or(intent.take());
        }
    });
    intent
}
