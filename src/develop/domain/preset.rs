use serde::{Deserialize, Serialize};

use super::adjustments::Adjustments;
use super::edit::FULL_INTENSITY;
use super::white_balance::WhiteBalance;
use super::zone::Zone;

/// A retouch made in one click: the zones it masks and what it sets on each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    WhiterTeeth,
    EnhancedSky,
    BrightEyes,
    SubjectPop,
    SkinGlow,
    LipColor,
    HairShine,
    DramaticSky,
    GoldenSky,
}

/// What the presets of a group retouch, as the Presets section sorts them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetGroup {
    Portrait,
    Scene,
}

impl PresetGroup {
    pub const ALL: [Self; 2] = [Self::Portrait, Self::Scene];

    pub fn name(self) -> &'static str {
        match self {
            Self::Portrait => "Portrait",
            Self::Scene => "Scene",
        }
    }

    pub fn presets(self) -> &'static [Preset] {
        match self {
            Self::Portrait => &[
                Preset::WhiterTeeth,
                Preset::BrightEyes,
                Preset::SkinGlow,
                Preset::LipColor,
                Preset::HairShine,
            ],
            Self::Scene => &[
                Preset::EnhancedSky,
                Preset::DramaticSky,
                Preset::GoldenSky,
                Preset::SubjectPop,
            ],
        }
    }
}

impl Preset {
    pub fn name(self) -> &'static str {
        match self {
            Self::WhiterTeeth => "Whiter teeth",
            Self::EnhancedSky => "Enhanced sky",
            Self::BrightEyes => "Bright eyes",
            Self::SubjectPop => "Subject pop",
            Self::SkinGlow => "Skin glow",
            Self::LipColor => "Lip color",
            Self::HairShine => "Hair shine",
            Self::DramaticSky => "Dramatic sky",
            Self::GoldenSky => "Golden sky",
        }
    }

    /// The zones the preset masks, each with the adjustments its mask gets.
    pub fn zone_adjustments(self) -> Vec<(Zone, Adjustments)> {
        match self {
            Self::WhiterTeeth => vec![(
                Zone::Teeth,
                Adjustments {
                    exposure: 0.4,
                    saturation: -60.0,
                    ..Adjustments::default()
                },
            )],
            Self::EnhancedSky => vec![(
                Zone::Sky,
                Adjustments {
                    exposure: -0.2,
                    contrast: 20.0,
                    highlights: -40.0,
                    vibrance: 30.0,
                    ..Adjustments::default()
                },
            )],
            Self::BrightEyes => vec![(
                Zone::Eyes,
                Adjustments {
                    exposure: 0.25,
                    contrast: 15.0,
                    whites: 10.0,
                    ..Adjustments::default()
                },
            )],
            Self::SkinGlow => vec![(
                Zone::Skin,
                Adjustments {
                    exposure: 0.15,
                    contrast: -10.0,
                    ..Adjustments::default()
                },
            )],
            Self::LipColor => vec![(
                Zone::Lips,
                Adjustments {
                    vibrance: 20.0,
                    saturation: 20.0,
                    ..Adjustments::default()
                },
            )],
            Self::HairShine => vec![(
                Zone::Hair,
                Adjustments {
                    contrast: 15.0,
                    whites: 15.0,
                    ..Adjustments::default()
                },
            )],
            Self::DramaticSky => vec![(
                Zone::Sky,
                Adjustments {
                    exposure: -0.6,
                    contrast: 40.0,
                    highlights: -60.0,
                    blacks: -15.0,
                    ..Adjustments::default()
                },
            )],
            Self::GoldenSky => vec![(
                Zone::Sky,
                Adjustments {
                    white_balance: Some(WhiteBalance {
                        temperature: 30.0,
                        tint: 5.0,
                    }),
                    exposure: -0.1,
                    vibrance: 25.0,
                    saturation: 10.0,
                    ..Adjustments::default()
                },
            )],
            Self::SubjectPop => vec![
                (
                    Zone::Subject,
                    Adjustments {
                        exposure: 0.2,
                        contrast: 10.0,
                        ..Adjustments::default()
                    },
                ),
                (
                    Zone::Background,
                    Adjustments {
                        exposure: -0.3,
                        saturation: -15.0,
                        ..Adjustments::default()
                    },
                ),
            ],
        }
    }

    pub fn mask_count(self) -> usize {
        self.zone_adjustments().len()
    }
}

/// What a preset made on one photo, carried by each of the masks it made there.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AppliedPreset {
    /// Tells apart the applied presets of one photo.
    pub id: u32,
    pub preset: Preset,
    /// How much of the effect of its masks is kept, 0 … 100.
    pub intensity: f32,
}

impl AppliedPreset {
    pub fn of(preset: Preset, id: u32) -> Self {
        Self {
            id,
            preset,
            intensity: FULL_INTENSITY,
        }
    }

    /// The share of the effect of its masks that is kept, 0 … 1.
    pub fn effect_share(&self) -> f32 {
        (self.intensity / FULL_INTENSITY).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_preset_keeps_all_of_its_effect_until_it_is_dosed() {
        let applied = AppliedPreset::of(Preset::EnhancedSky, 0);
        assert_eq!(applied.effect_share(), 1.0);

        let halved = AppliedPreset {
            intensity: 50.0,
            ..applied
        };
        assert_eq!(halved.effect_share(), 0.5);
    }

    #[test]
    fn every_preset_is_in_one_group_under_its_own_name() {
        let grouped: Vec<Preset> = PresetGroup::ALL
            .iter()
            .flat_map(|group| group.presets().iter().copied())
            .collect();
        let mut names: Vec<&str> = grouped.iter().map(|preset| preset.name()).collect();
        names.sort_unstable();
        names.dedup();

        assert_eq!(grouped.len(), 9);
        assert_eq!(names.len(), 9);
    }
}
