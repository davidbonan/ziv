use crate::design::ui::adjustment_slider::{AdjustmentSlider, Track, TrackScale};
use crate::design::ui::marked_choice::{MarkedChoice, marked_choice, marked_choice_label};
use crate::design::ui::theme::{color, regular, space, type_size};
use crate::develop::domain::adjustments::ADJUSTMENT_RANGE;
use crate::develop::domain::color_grading::{
    BLENDING_RANGE, ColorGrading, DEFAULT_BLENDING, GRADE_SATURATION_RANGE, HUE_RANGE, TonalZone,
    ZoneGrade,
};

use super::color_wheel::{Wheel, color_wheel};

pub const GRADE_HUE_LABEL: &str = "Hue";
pub const GRADE_SATURATION_LABEL: &str = "Saturation";
pub const GRADE_LUMINANCE_LABEL: &str = "Luminance";
pub const BLENDING_LABEL: &str = "Blending";
pub const BALANCE_LABEL: &str = "Balance";
pub const THREE_WAY_LABEL: &str = "3-way grading";

const LARGEST_WHEEL: f32 = 168.0;
const THREE_WAY_ZONES: [TonalZone; 3] = [
    TonalZone::Shadows,
    TonalZone::Midtones,
    TonalZone::Highlights,
];

const fn whole_number_slider(
    label: &'static str,
    range: std::ops::RangeInclusive<f32>,
) -> AdjustmentSlider {
    AdjustmentSlider {
        label,
        range,
        default: 0.0,
        step: 1.0,
        decimals: 0,
        scale: TrackScale::Linear,
        track: Track::AccentFill,
    }
}

const HUE: AdjustmentSlider = whole_number_slider(GRADE_HUE_LABEL, HUE_RANGE);
const SATURATION: AdjustmentSlider =
    whole_number_slider(GRADE_SATURATION_LABEL, GRADE_SATURATION_RANGE);
const LUMINANCE: AdjustmentSlider = whole_number_slider(GRADE_LUMINANCE_LABEL, ADJUSTMENT_RANGE);
const BLENDING: AdjustmentSlider = AdjustmentSlider {
    default: DEFAULT_BLENDING,
    ..whole_number_slider(BLENDING_LABEL, BLENDING_RANGE)
};
const BALANCE: AdjustmentSlider = whole_number_slider(BALANCE_LABEL, ADJUSTMENT_RANGE);

/// What the section shows of the grading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum GradingView {
    /// The wheels of shadows, midtones and highlights together.
    #[default]
    ThreeWay,
    Zone(TonalZone),
}

fn zone_name(zone: TonalZone) -> &'static str {
    match zone {
        TonalZone::Shadows => "Shadows",
        TonalZone::Midtones => "Midtones",
        TonalZone::Highlights => "Highlights",
        TonalZone::Global => "Global",
    }
}

pub fn zone_wheel_label(zone: TonalZone) -> &'static str {
    match zone {
        TonalZone::Shadows => "Shadows wheel",
        TonalZone::Midtones => "Midtones wheel",
        TonalZone::Highlights => "Highlights wheel",
        TonalZone::Global => "Global wheel",
    }
}

/// The Luminance slider of a zone in the 3-way view.
pub fn zone_luminance_label(zone: TonalZone) -> &'static str {
    match zone {
        TonalZone::Shadows => "Shadows luminance",
        TonalZone::Midtones => "Midtones luminance",
        TonalZone::Highlights => "Highlights luminance",
        TonalZone::Global => "Global luminance",
    }
}

fn zone_view_label(zone: TonalZone) -> String {
    format!("{} grading", zone_name(zone))
}

/// What assistive technology and tests read for a zone of the selector.
pub fn grading_zone_label(zone: TonalZone, is_edited: bool) -> String {
    marked_choice_label(&zone_view_label(zone), is_edited)
}

/// 3-way, then the four zones. Returns the view to show.
fn view_selector(ui: &mut egui::Ui, grading: &ColorGrading, shown: GradingView) -> GradingView {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(space::XS, space::XS);
        let three_way = MarkedChoice {
            text: "3-way",
            name: THREE_WAY_LABEL,
            is_chosen: shown == GradingView::ThreeWay,
            is_edited: false,
        };
        let three_way_asked = marked_choice(ui, &three_way)
            .clicked()
            .then_some(GradingView::ThreeWay);
        let zone_asked = TonalZone::ALL.into_iter().filter(|zone| {
            let choice = MarkedChoice {
                text: zone_name(*zone),
                name: &zone_view_label(*zone),
                is_chosen: shown == GradingView::Zone(*zone),
                is_edited: !grading.of(*zone).is_default(),
            };
            marked_choice(ui, &choice).clicked()
        });
        zone_asked
            .last()
            .map(GradingView::Zone)
            .or(three_way_asked)
            .unwrap_or(shown)
    })
    .inner
}

/// One large wheel and the three sliders of its zone, showing the same values.
fn zone_view(ui: &mut egui::Ui, zone: TonalZone, grade: ZoneGrade) -> ZoneGrade {
    let wheel = Wheel {
        name: zone_wheel_label(zone),
        diameter: ui.available_width().min(LARGEST_WHEEL),
    };
    let placed = ui
        .vertical_centered(|ui| color_wheel(ui, &wheel, grade))
        .inner;
    ZoneGrade {
        hue: HUE.show(ui, placed.hue),
        saturation: SATURATION.show(ui, placed.saturation),
        luminance: LUMINANCE.show(ui, placed.luminance),
    }
}

/// The three wheels side by side, each over its name, then their Luminance sliders.
fn three_way_view(ui: &mut egui::Ui, grading: ColorGrading) -> ColorGrading {
    let gap = space::S;
    let diameter = (ui.available_width() - 2.0 * gap) / THREE_WAY_ZONES.len() as f32;
    let placed = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            THREE_WAY_ZONES.map(|zone| {
                let wheel = Wheel {
                    name: zone_wheel_label(zone),
                    diameter,
                };
                ui.vertical(|ui| {
                    let grade = color_wheel(ui, &wheel, *grading.of(zone));
                    let name = egui::RichText::new(zone_name(zone))
                        .font(regular(type_size::CAPTION))
                        .color(color::TEXT_MUTED);
                    ui.allocate_ui_with_layout(
                        egui::vec2(diameter, 0.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| ui.label(name),
                    );
                    grade
                })
                .inner
            })
        })
        .inner;
    THREE_WAY_ZONES
        .into_iter()
        .zip(placed)
        .fold(grading, |grading, (zone, grade)| {
            let slider = AdjustmentSlider {
                label: zone_luminance_label(zone),
                ..LUMINANCE
            };
            let lit = ZoneGrade {
                luminance: slider.show(ui, grade.luminance),
                ..grade
            };
            grading.with(zone, lit)
        })
}

/// The view selector, the view chosen, then Blending and Balance. Returns
/// the color grading as the user left it this frame.
pub fn color_grading_section(ui: &mut egui::Ui, grading: &ColorGrading) -> ColorGrading {
    let shown_id = ui.id().with("grading view shown");
    let shown: Option<GradingView> = ui.data(|data| data.get_temp(shown_id));
    let shown = view_selector(ui, grading, shown.unwrap_or_default());
    ui.data_mut(|data| data.insert_temp(shown_id, shown));

    let graded = match shown {
        GradingView::ThreeWay => three_way_view(ui, *grading),
        GradingView::Zone(zone) => grading.with(zone, zone_view(ui, zone, *grading.of(zone))),
    };
    ColorGrading {
        blending: BLENDING.show(ui, graded.blending),
        balance: BALANCE.show(ui, graded.balance),
        ..graded
    }
}
