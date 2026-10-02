use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::white_balance::WhiteBalance;

pub const EXPOSURE_RANGE: RangeInclusive<f32> = -5.0..=5.0;
pub const ADJUSTMENT_RANGE: RangeInclusive<f32> = -100.0..=100.0;

/// The adjustments of one photo. Every default leaves the photo unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Adjustments {
    /// `None`: as shot.
    pub white_balance: Option<WhiteBalance>,
    /// In EV: +1 doubles the light.
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    pub vibrance: f32,
    pub saturation: f32,
}

impl Adjustments {
    pub fn exposure_gain(&self) -> f32 {
        self.exposure.exp2()
    }
}
