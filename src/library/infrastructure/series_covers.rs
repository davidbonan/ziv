use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::library::domain::catalog::Catalog;
use crate::library::domain::series::{COVER_PHOTO_COUNT, Series};
use crate::photo::application::photo_loader::{Backlog, LoadedPhoto, PhotoLoader};
use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::thumbnail::Thumbnail;

use super::thumbnail_texture::thumbnail_texture;

/// The thumbnails standing for the series in the sidebar, loaded on a worker thread.
pub struct SeriesCovers {
    egui_context: egui::Context,
    loader: PhotoLoader<Thumbnail>,
    /// `None` while a thumbnail loads, and for good when it could not be made.
    textures: HashMap<PathBuf, Option<egui::TextureHandle>>,
}

impl SeriesCovers {
    pub fn spawn(
        egui_context: &egui::Context,
        load: impl Fn(&Path) -> Result<Thumbnail, DecodeError> + Send + 'static,
    ) -> Self {
        let repainting_context = egui_context.clone();
        Self {
            egui_context: egui_context.clone(),
            loader: PhotoLoader::spawn(Backlog::LoadAll, load, move || {
                repainting_context.request_repaint()
            }),
            textures: HashMap::new(),
        }
    }

    /// Asks for the cover photos of `catalog` that were never asked for.
    pub fn request_those_of(&mut self, catalog: &Catalog) {
        let cover_photos = catalog.series().iter().flat_map(Series::cover_photos);
        for photo in cover_photos {
            if !self.textures.contains_key(photo) {
                self.textures.insert(photo.clone(), None);
                self.loader.request(photo.clone());
            }
        }
    }

    pub fn receive_loaded(&mut self) {
        for LoadedPhoto { path, result } in self.loader.take_loaded() {
            if let Ok(thumbnail) = result {
                let texture = thumbnail_texture(&self.egui_context, &path, &thumbnail);
                self.textures.insert(path, Some(texture));
            }
        }
    }

    pub fn cover_of(&self, series: &Series) -> [Option<&egui::TextureHandle>; COVER_PHOTO_COUNT] {
        let mut photos = series.cover_photos().iter();
        std::array::from_fn(|_| self.textures.get(photos.next()?)?.as_ref())
    }
}
