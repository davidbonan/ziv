use std::path::Path;

use trash::TrashContext;

use crate::library::domain::photo_trash::PhotoTrash;

/// The Trash of this computer.
pub struct SystemTrash;

// Through the file manager: asking the Finder needs the user to let ziv control it.
#[cfg(target_os = "macos")]
fn trash_context() -> TrashContext {
    use trash::macos::{DeleteMethod, TrashContextExtMacos};

    let mut context = TrashContext::default();
    context.set_delete_method(DeleteMethod::NsFileManager);
    context
}

#[cfg(not(target_os = "macos"))]
fn trash_context() -> TrashContext {
    TrashContext::default()
}

impl PhotoTrash for SystemTrash {
    fn move_to_trash(&self, file: &Path) -> Result<(), String> {
        trash_context()
            .delete(file)
            .map_err(|error| error.to_string())
    }
}
