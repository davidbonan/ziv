use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ort::ep::CoreML;
use ort::ep::coreml::{ComputeUnits, ModelFormat};
use ort::session::Session;
use ort::session::builder::SessionBuilder;
use ort::value::Tensor;

use crate::models::domain::model_runner::{FixedDimension, ModelInput, ModelOutput, ModelRunner};

/// Runs models with ONNX Runtime; each model stays loaded as long as the
/// runner lives.
#[derive(Default)]
pub struct OnnxRunner {
    sessions: Mutex<HashMap<PathBuf, Session>>,
    /// Not empty: the models run on the GPU, which takes fixed shapes only.
    fixed_dimensions: &'static [FixedDimension],
}

fn outputs_of(session: &mut Session, input: ModelInput) -> ort::Result<Vec<ModelOutput>> {
    let tensor = Tensor::from_array((input.shape, input.values))?;
    let outputs = session.run(ort::inputs![input.name => tensor])?;
    outputs
        .iter()
        .map(|(_, value)| {
            let (shape, values) = value.try_extract_tensor::<f32>()?;
            Ok(ModelOutput {
                shape: shape.iter().map(|side| *side as usize).collect(),
                values: values.to_vec(),
            })
        })
        .collect()
}

impl OnnxRunner {
    /// Runs on the GPU the models whose free dimensions are these; on the
    /// processor when the GPU cannot take them.
    pub fn on_gpu(fixed_dimensions: &'static [FixedDimension]) -> Self {
        Self {
            sessions: Mutex::default(),
            fixed_dimensions,
        }
    }

    fn on_gpu_with_fixed_shapes(&self, builder: SessionBuilder) -> ort::Result<SessionBuilder> {
        let mut builder = builder;
        for (name, size) in self.fixed_dimensions {
            builder = builder.with_dimension_override(*name, *size)?;
        }
        let gpu = CoreML::default()
            .with_model_format(ModelFormat::MLProgram)
            .with_static_input_shapes(true)
            .with_compute_units(ComputeUnits::CPUAndGPU);
        Ok(builder.with_execution_providers([gpu.build()])?)
    }

    fn loaded(&self, model_file: &Path) -> ort::Result<Session> {
        let mut builder = Session::builder()?;
        if !self.fixed_dimensions.is_empty() {
            builder = self.on_gpu_with_fixed_shapes(builder)?;
        }
        builder.commit_from_file(model_file)
    }
}

impl ModelRunner for OnnxRunner {
    fn outputs(&self, model_file: &Path, input: ModelInput) -> Result<Vec<ModelOutput>, String> {
        let mut sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let session = match sessions.entry(model_file.to_owned()) {
            Entry::Occupied(loaded) => loaded.into_mut(),
            Entry::Vacant(missing) => {
                let loaded = self.loaded(model_file).map_err(|error| error.to_string())?;
                missing.insert(loaded)
            }
        };
        outputs_of(session, input).map_err(|error| error.to_string())
    }
}
