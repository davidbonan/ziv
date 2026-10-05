use crate::design::ui::adjustment_slider::{AdjustmentSlider, Track, TrackScale};
use crate::design::ui::icon_button::{Icon, icon_button};
use crate::design::ui::marked_choice::{MarkedChoice, marked_choice};
use crate::design::ui::theme::space;
use crate::develop::domain::aspect_ratio::{NamedRatio, RatioLock};
use crate::develop::domain::framing::{ANGLE_RANGE, Framing, Turn};

use super::develop_panel::title_with_done;

pub const CROP_GROUP_LABEL: &str = "Crop";
pub const ANGLE_LABEL: &str = "Angle";
pub const LOCK_RATIO_LABEL: &str = "Lock ratio";
pub const LEVEL_LABEL: &str = "Level";
pub const ROTATE_LEFT_LABEL: &str = "Rotate left";
pub const ROTATE_RIGHT_LABEL: &str = "Rotate right";
pub const FLIP_HORIZONTAL_LABEL: &str = "Flip horizontal";
pub const FLIP_VERTICAL_LABEL: &str = "Flip vertical";
pub const RESET_CROP_LABEL: &str = "Reset crop";

const ANGLE: AdjustmentSlider = AdjustmentSlider {
    label: ANGLE_LABEL,
    range: ANGLE_RANGE,
    default: 0.0,
    step: 0.1,
    decimals: 1,
    scale: TrackScale::Linear,
    track: Track::AccentFill,
};

/// What assistive technology and tests read for a choice of the ratio selector.
pub fn ratio_label(ratio: RatioLock) -> String {
    format!("{} ratio", ratio.name())
}

/// What the user asked of the proportions of the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatioAsked {
    Free,
    /// The frame takes these proportions and keeps them.
    Named(NamedRatio),
    /// The frame keeps the proportions it has.
    OfFrame,
}

/// What the frame is being worked on with, in crop mode.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CropTools {
    /// What holds the proportions of the frame.
    pub ratio: RatioLock,
    /// The next drag on the picture draws the line to level.
    pub is_level_armed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropSectionShown {
    pub framing: Framing,
    pub tools: CropTools,
}

pub struct CropSectionOutput {
    /// The framing as the section left it this frame.
    pub framing: Framing,
    pub ratio_asked: Option<RatioAsked>,
    pub is_level_armed: bool,
    pub is_done: bool,
}

/// Free, then the named ratios; Custom only while the frame is locked on it.
fn ratio_selector(ui: &mut egui::Ui, shown: RatioLock) -> Option<RatioAsked> {
    let free = (RatioLock::Free, RatioAsked::Free);
    let named = NamedRatio::ALL.map(|named| (RatioLock::Named(named), RatioAsked::Named(named)));
    let custom = (RatioLock::Custom, RatioAsked::OfFrame);
    let choices = std::iter::once(free)
        .chain(named)
        .chain((shown == RatioLock::Custom).then_some(custom));
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(space::XS, space::XS);
        let clicked = choices.filter(|(ratio, _)| {
            let choice = MarkedChoice {
                text: ratio.name(),
                name: &ratio_label(*ratio),
                is_chosen: *ratio == shown,
                is_edited: false,
            };
            marked_choice(ui, &choice).clicked()
        });
        clicked.map(|(_, asked)| asked).last()
    })
    .inner
}

/// The lock keeps the proportions the frame has; asked again, it lets them go.
fn ratio_lock(ui: &mut egui::Ui, shown: RatioLock) -> Option<RatioAsked> {
    let lock = egui::Button::new(LOCK_RATIO_LABEL).selected(shown.is_locked());
    let asked = match shown.is_locked() {
        true => RatioAsked::Free,
        false => RatioAsked::OfFrame,
    };
    ui.add(lock).clicked().then_some(asked)
}

struct TurnButton {
    icon: Icon,
    label: &'static str,
    turn: fn(Turn) -> Turn,
}

const TURN_BUTTONS: [TurnButton; 4] = [
    TurnButton {
        icon: Icon::RotateLeft,
        label: ROTATE_LEFT_LABEL,
        turn: Turn::turned_left,
    },
    TurnButton {
        icon: Icon::RotateRight,
        label: ROTATE_RIGHT_LABEL,
        turn: Turn::turned_right,
    },
    TurnButton {
        icon: Icon::FlipHorizontal,
        label: FLIP_HORIZONTAL_LABEL,
        turn: Turn::flipped_horizontally,
    },
    TurnButton {
        icon: Icon::FlipVertical,
        label: FLIP_VERTICAL_LABEL,
        turn: Turn::flipped_vertically,
    },
];

/// The quarter turns and the mirrors, one row of icons.
fn turned(ui: &mut egui::Ui, shown: Turn) -> Turn {
    ui.horizontal(|ui| {
        let asked = TURN_BUTTONS
            .iter()
            .filter(|button| icon_button(ui, button.icon, button.label).clicked())
            .last();
        asked.map_or(shown, |button| (button.turn)(shown))
    })
    .inner
}

/// The Crop section, shown in place of the photo's while its framing is edited.
pub fn crop_section(ui: &mut egui::Ui, shown: &CropSectionShown) -> CropSectionOutput {
    let is_done = title_with_done(ui, CROP_GROUP_LABEL);
    let CropTools {
        ratio,
        is_level_armed,
    } = shown.tools;
    let chosen = ratio_selector(ui, ratio);
    let locked = ratio_lock(ui, ratio);
    ui.add_space(space::S);
    // Shown as the photo is seen to turn: a mirror turns it the other way.
    let angle_sense = shown.framing.turn.angle_sense();
    let angle = ANGLE.show(ui, shown.framing.angle * angle_sense) * angle_sense;
    let level = egui::Button::new(LEVEL_LABEL).selected(is_level_armed);
    let is_level_asked = ui.add(level).clicked();
    ui.add_space(space::S);
    let turn = turned(ui, shown.framing.turn);
    ui.add_space(space::S);
    let reset_crop = egui::Button::new(RESET_CROP_LABEL);
    let is_reset_asked = ui
        .add_enabled(shown.framing.is_cropped(), reset_crop)
        .clicked();
    let framing = Framing {
        angle,
        turn,
        ..shown.framing
    };
    // A held ratio is that of the frame: once the frame is the picture's, so is the ratio.
    let held_again = (is_reset_asked && ratio.is_locked()).then_some(RatioAsked::OfFrame);
    CropSectionOutput {
        framing: match is_reset_asked {
            true => framing.without_crop(),
            false => framing,
        },
        ratio_asked: chosen.or(locked).or(held_again),
        is_level_armed: is_level_armed != is_level_asked,
        is_done,
    }
}
