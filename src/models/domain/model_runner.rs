use std::path::Path;

pub struct ModelInput {
    pub name: &'static str,
    pub shape: [usize; 4],
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelOutput {
    pub shape: Vec<usize>,
    pub values: Vec<f32>,
}

/// The size given to a dimension a model leaves free, by the name the model gives it.
pub type FixedDimension = (&'static str, i64);

/// What computes the outputs of a model.
pub trait ModelRunner {
    fn outputs(&self, model_file: &Path, input: ModelInput) -> Result<Vec<ModelOutput>, String>;
}
