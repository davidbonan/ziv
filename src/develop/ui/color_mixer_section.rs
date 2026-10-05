use egui::Color32;

use crate::color::domain::working_space::DisplayTransform;
use crate::design::ui::adjustment_slider::{AdjustmentSlider, Track, TrackScale};
use crate::design::ui::marked_choice::{MarkedChoice, marked_choice, marked_choice_label};
use crate::design::ui::theme::space;
use crate::develop::domain::adjustments::ADJUSTMENT_RANGE;
use crate::develop::domain::color_mixer::{ColorMixer, ColorRange, MixerAdjustment};

fn adjustment_name(adjustment: MixerAdjustment) -> &'static str {
    match adjustment {
        MixerAdjustment::Hue => "Hue",
        MixerAdjustment::Saturation => "Saturation",
        MixerAdjustment::Luminance => "Luminance",
    }
}

pub fn color_range_label(range: ColorRange) -> &'static str {
    match range {
        ColorRange::Red => "Red",
        ColorRange::Orange => "Orange",
        ColorRange::Yellow => "Yellow",
        ColorRange::Green => "Green",
        ColorRange::Aqua => "Aqua",
        ColorRange::Blue => "Blue",
        ColorRange::Purple => "Purple",
        ColorRange::Magenta => "Magenta",
    }
}

fn adjustment_label(adjustment: MixerAdjustment) -> String {
    format!("{} mixer", adjustment_name(adjustment))
}

/// What assistive technology and tests read for an adjustment of the selector.
pub fn mixer_adjustment_label(adjustment: MixerAdjustment, is_edited: bool) -> String {
    marked_choice_label(&adjustment_label(adjustment), is_edited)
}

/// The three adjustments in a row. Returns the adjustment to show.
fn adjustment_selector(
    ui: &mut egui::Ui,
    mixer: &ColorMixer,
    shown: MixerAdjustment,
) -> MixerAdjustment {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::XS;
        let clicked = MixerAdjustment::ALL.into_iter().filter(|adjustment| {
            let choice = MarkedChoice {
                text: adjustment_name(*adjustment),
                name: &adjustment_label(*adjustment),
                is_chosen: *adjustment == shown,
                is_edited: !mixer.of(*adjustment).is_default(),
            };
            marked_choice(ui, &choice).clicked()
        });
        clicked.last().unwrap_or(shown)
    })
    .inner
}

fn on_screen(display: &DisplayTransform, working: [f32; 3]) -> Color32 {
    let [red, green, blue] = display
        .to_display(working)
        .map(|channel| (channel * 255.0).round() as u8);
    Color32::from_rgb(red, green, blue)
}

/// What each end of an adjustment makes of a plain color of the range.
fn track_colors(
    display: &DisplayTransform,
    adjustment: MixerAdjustment,
    range: ColorRange,
) -> [Color32; 3] {
    let sample = range.sample();
    [*ADJUSTMENT_RANGE.start(), 0.0, *ADJUSTMENT_RANGE.end()].map(|value| {
        let mut values = *ColorMixer::default().of(adjustment);
        values[range] = value;
        let mixer = ColorMixer::default().with(adjustment, values);
        on_screen(display, mixer.applied(sample))
    })
}

/// The Hue, Saturation or Luminance selector and the eight sliders of the
/// adjustment shown. Returns the color mixer as the user left it this frame.
pub fn color_mixer_section(ui: &mut egui::Ui, mixer: &ColorMixer) -> ColorMixer {
    let shown_id = ui.id().with("mixer adjustment shown");
    let shown: Option<MixerAdjustment> = ui.data(|data| data.get_temp(shown_id));
    let shown = adjustment_selector(ui, mixer, shown.unwrap_or_default());
    ui.data_mut(|data| data.insert_temp(shown_id, shown));

    let display = DisplayTransform::default();
    let mut values = *mixer.of(shown);
    for range in ColorRange::ALL {
        let slider = AdjustmentSlider {
            label: color_range_label(range),
            range: ADJUSTMENT_RANGE,
            default: 0.0,
            step: 1.0,
            decimals: 0,
            scale: TrackScale::Linear,
            track: Track::Colors(track_colors(&display, shown, range)),
        };
        values[range] = slider.show(ui, values[range]);
    }
    mixer.with(shown, values)
}
