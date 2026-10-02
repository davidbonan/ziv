use std::path::Path;

use crate::photo::domain::photo_name::photo_name;
use crate::photo::domain::thumbnail::Thumbnail;

pub fn thumbnail_texture(
    egui_context: &egui::Context,
    photo: &Path,
    thumbnail: &Thumbnail,
) -> egui::TextureHandle {
    let size = [thumbnail.width as usize, thumbnail.height as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, &thumbnail.rgba);
    egui_context.load_texture(photo_name(photo), image, egui::TextureOptions::LINEAR)
}
