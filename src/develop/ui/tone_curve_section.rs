use egui::Color32;

use crate::design::ui::marked_choice::{MarkedChoice, marked_choice, marked_choice_label};
use crate::design::ui::theme::{color, hue, space};
use crate::develop::domain::tone_curve::{CurveChannel, ToneCurve, ToneCurves};

use crate::histogram::domain::histogram::Histogram;

use super::tone_curve_graph::{GraphShown, tone_curve_graph};

pub const RESET_CURVE_LABEL: &str = "Reset curve";

fn channel_name(channel: CurveChannel) -> &'static str {
    match channel {
        CurveChannel::Rgb => "RGB",
        CurveChannel::Red => "Red",
        CurveChannel::Green => "Green",
        CurveChannel::Blue => "Blue",
    }
}

fn channel_ink(channel: CurveChannel) -> Color32 {
    match channel {
        CurveChannel::Rgb => color::TEXT,
        CurveChannel::Red => hue::RED,
        CurveChannel::Green => hue::GREEN,
        CurveChannel::Blue => hue::BLUE,
    }
}

fn channel_label(channel: CurveChannel) -> String {
    format!("{} curve", channel_name(channel))
}

/// What assistive technology and tests read for a channel of the selector.
pub fn curve_channel_label(channel: CurveChannel, is_edited: bool) -> String {
    marked_choice_label(&channel_label(channel), is_edited)
}

/// The four channels in a row. Returns the channel to show.
fn channel_selector(ui: &mut egui::Ui, curves: &ToneCurves, shown: CurveChannel) -> CurveChannel {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::XS;
        let clicked = CurveChannel::ALL.into_iter().filter(|channel| {
            let choice = MarkedChoice {
                text: channel_name(*channel),
                name: &channel_label(*channel),
                is_chosen: *channel == shown,
                is_edited: !curves.of(*channel).is_identity(),
            };
            marked_choice(ui, &choice).clicked()
        });
        clicked.last().unwrap_or(shown)
    })
    .inner
}

/// The channel selector, the graph of the channel shown over `histogram`,
/// and Reset curve. Returns the curves as the user left them this frame.
pub fn tone_curve_section(
    ui: &mut egui::Ui,
    curves: &ToneCurves,
    histogram: Option<&Histogram>,
) -> ToneCurves {
    let shown_id = ui.id().with("curve channel shown");
    let shown: Option<CurveChannel> = ui.data(|data| data.get_temp(shown_id));
    let shown = channel_selector(ui, curves, shown.unwrap_or_default());
    ui.data_mut(|data| data.insert_temp(shown_id, shown));

    let graph = GraphShown {
        curve: curves.of(shown),
        ink: channel_ink(shown),
        histogram,
    };
    let curve = tone_curve_graph(ui, &graph);
    let reset = egui::Button::new(RESET_CURVE_LABEL);
    let is_reset_asked = ui.add_enabled(!curve.is_identity(), reset).clicked();
    let curve = match is_reset_asked {
        true => ToneCurve::default(),
        false => curve,
    };
    curves.clone().with(shown, curve)
}
