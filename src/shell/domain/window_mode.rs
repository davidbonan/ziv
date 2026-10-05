use serde::{Deserialize, Serialize};

/// What the window is laid out for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WindowMode {
    /// Sorting a series: grid and preview.
    Cull,
    #[default]
    Develop,
}
