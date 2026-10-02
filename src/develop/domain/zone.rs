use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::coverage_image::CoverageImage;

/// What a zone mask covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Subject,
    Background,
    Sky,
    Skin,
    Hair,
    Eyes,
    Lips,
    Clothes,
}

impl Zone {
    pub fn name(self) -> &'static str {
        match self {
            Self::Subject => "Subject",
            Self::Background => "Background",
            Self::Sky => "Sky",
            Self::Skin => "Skin",
            Self::Hair => "Hair",
            Self::Eyes => "Eyes",
            Self::Lips => "Lips",
            Self::Clothes => "Clothes",
        }
    }
}

/// What can be masked of a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonPart {
    Skin,
    Hair,
    Eyes,
    Lips,
    Clothes,
}

impl PersonPart {
    pub fn name(self) -> &'static str {
        self.zone().name()
    }

    pub const ALL: [Self; 5] = [
        Self::Skin,
        Self::Hair,
        Self::Eyes,
        Self::Lips,
        Self::Clothes,
    ];

    pub fn zone(self) -> Zone {
        match self {
            Self::Skin => Zone::Skin,
            Self::Hair => Zone::Hair,
            Self::Eyes => Zone::Eyes,
            Self::Lips => Zone::Lips,
            Self::Clothes => Zone::Clothes,
        }
    }
}

/// What the user asks to be detected in the photo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneTool {
    Subject,
    Background,
    Sky,
}

/// What the user asks of the zone tools: a zone of the photo, or its persons
/// to choose parts of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionTool {
    Zone(ZoneTool),
    People,
}

/// A mask whose coverage was detected in the photo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneMask {
    pub zone: Zone,
    pub coverage: Arc<CoverageImage>,
}
