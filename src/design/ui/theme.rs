use std::sync::Arc;

use egui::epaint::text::{FontData, FontDefinitions};
use egui::style::{Selection, WidgetVisuals};
use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, vec2};

/// Surfaces and text are neutral greys: nothing around a photo tints how its
/// colors are judged. The accent is the only hue of the interface.
pub mod color {
    use egui::Color32;

    /// Around the photo.
    pub const CANVAS: Color32 = Color32::from_gray(0x1B);
    /// Panels beside and below the photo.
    pub const PANEL: Color32 = Color32::from_gray(0x26);
    /// Buttons, fields, thumbnail cells.
    pub const RAISED: Color32 = Color32::from_gray(0x33);
    pub const RAISED_HOVERED: Color32 = Color32::from_gray(0x3D);
    pub const TRACK: Color32 = Color32::from_gray(0x47);
    /// The groove between two surfaces.
    pub const HAIRLINE: Color32 = Color32::from_gray(0x12);
    pub const TEXT: Color32 = Color32::from_gray(0xE4);
    pub const TEXT_MUTED: Color32 = Color32::from_gray(0x9B);
    pub const TEXT_DISABLED: Color32 = Color32::from_gray(0x5C);
    /// Darkroom safelight amber: selection, and values away from their default.
    pub const ACCENT: Color32 = Color32::from_rgb(0xF0, 0xA2, 0x3B);
    pub const DANGER: Color32 = Color32::from_rgb(0xE0, 0x70, 0x5A);
    /// Pure white at low opacity: a tinted outline reads as dirt on a picture's edge.
    pub const PICTURE_OUTLINE: Color32 = Color32::from_rgba_premultiplied(26, 26, 26, 26);
}

pub mod space {
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
}

pub mod type_size {
    pub const CAPTION: f32 = 11.0;
    pub const BODY: f32 = 13.0;
    pub const TITLE: f32 = 15.0;
}

pub const CONTROL_RADIUS: u8 = 4;

const REGULAR: &str = "IBM Plex Sans";
const MEDIUM: &str = "IBM Plex Sans Medium";

/// The heavier cut, for names and titles.
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MEDIUM.into()))
}

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let regular = include_bytes!("../../../assets/fonts/IBMPlexSans-Regular.ttf");
    let medium = include_bytes!("../../../assets/fonts/IBMPlexSans-Medium.ttf");
    fonts
        .font_data
        .insert(REGULAR.to_owned(), Arc::new(FontData::from_static(regular)));
    fonts
        .font_data
        .insert(MEDIUM.to_owned(), Arc::new(FontData::from_static(medium)));

    // Plex first, egui's own fonts kept behind it for the symbols Plex lacks.
    let mut fallbacks = fonts.families[&FontFamily::Proportional].clone();
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .expect("egui defines the proportional family")
        .insert(0, REGULAR.to_owned());
    fallbacks.insert(0, MEDIUM.to_owned());
    fonts
        .families
        .insert(FontFamily::Name(MEDIUM.into()), fallbacks);
    fonts
}

fn widget(fill: Color32, text: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::NONE,
        corner_radius: CornerRadius::same(CONTROL_RADIUS),
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    }
}

/// Makes `context` look like ziv: fonts, colors, spacing.
pub fn apply_theme(context: &egui::Context) {
    context.set_fonts(fonts());
    context.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Small, regular(type_size::CAPTION)),
            (TextStyle::Body, regular(type_size::BODY)),
            (TextStyle::Button, regular(type_size::BODY)),
            (TextStyle::Heading, medium(type_size::TITLE)),
            (TextStyle::Monospace, FontId::monospace(type_size::BODY)),
        ]
        .into();
        style.spacing.item_spacing = vec2(space::S, space::S);
        style.spacing.button_padding = vec2(space::M, 5.0);
        style.spacing.interact_size.y = 28.0;

        let visuals = &mut style.visuals;
        visuals.dark_mode = true;
        visuals.panel_fill = color::PANEL;
        visuals.window_fill = color::PANEL;
        visuals.extreme_bg_color = color::CANVAS;
        visuals.faint_bg_color = color::RAISED;
        visuals.warn_fg_color = color::DANGER;
        visuals.error_fg_color = color::DANGER;
        visuals.weak_text_color = Some(color::TEXT_MUTED);
        visuals.selection = Selection {
            bg_fill: color::TRACK,
            stroke: Stroke::new(1.0, color::ACCENT),
        };
        visuals.text_cursor.stroke = Stroke::new(1.5, color::TEXT);
        visuals.widgets.noninteractive = WidgetVisuals {
            bg_stroke: Stroke::new(1.0, color::HAIRLINE),
            ..widget(color::PANEL, color::TEXT)
        };
        visuals.widgets.inactive = widget(color::RAISED, color::TEXT);
        visuals.widgets.hovered = widget(color::RAISED_HOVERED, color::TEXT);
        visuals.widgets.active = widget(color::TRACK, color::TEXT);
        visuals.widgets.open = widget(color::RAISED_HOVERED, color::TEXT);
    });
}
