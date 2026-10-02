use crate::models::domain::model::{Model, RemoteFile};
use crate::models::domain::model_runner::FixedDimension;

use super::overlapping_tiles::TILE_SIDE;

/// DRUNet colour: a denoiser told how much noise to remove, made of
/// convolutions only, which a GPU runs fast.
pub const ENHANCEMENT_MODEL: Model = Model {
    name: "enhancement",
    file: RemoteFile {
        url: "https://huggingface.co/synthscript/drunet-color-onnx/resolve/a2b9fccfa27b197f44a3876c567f5e48970c44a7/drunet_color.onnx",
        sha256: "2ae3ab5eb15daac2ee79be984d584b908ce7f0f60b27be87d005f728c2aa0087",
        size: 130_589_232,
    },
};

pub const MODEL_INPUT: &str = "input";
/// The photo's three channels, then the noise level.
pub const INPUT_CHANNELS: usize = 4;
/// The noise levels the model was trained on, as standard deviations in its encoding.
pub const MOST_NOISE: f32 = 50.0 / 255.0;

/// The free dimensions of the model, fixed at one tile: what the GPU needs.
pub const TILE_SHAPE: [FixedDimension; 3] =
    [("b", 1), ("h", TILE_SIDE as i64), ("w", TILE_SIDE as i64)];
