use crate::models::domain::model::Model;

const BYTES_PER_MEGABYTE: u64 = 1_000_000;

fn megabytes(bytes: u64) -> u64 {
    bytes / BYTES_PER_MEGABYTE
}

/// How far the download of `model` is.
pub fn download_label(model: &Model, received: u64) -> String {
    let (received, size) = (megabytes(received), megabytes(model.file.size));
    format!(
        "Downloading the {} model, {received} of {size} MB",
        model.name
    )
}
