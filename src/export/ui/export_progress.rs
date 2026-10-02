use std::path::Path;

use egui::RichText;

use crate::design::ui::floating_pill::floating_pill;
use crate::design::ui::theme::{color, space};
use crate::export::application::export_run::ExportProgress;
use crate::photo::domain::photo_name::photo_name;

pub const STOP_EXPORT_LABEL: &str = "Cancel export";

pub fn progress_label(progress: &ExportProgress) -> String {
    let current = (progress.exported + progress.failed + 1).min(progress.total);
    format!("Exporting {current} of {}", progress.total)
}

fn photos(count: usize) -> String {
    match count {
        1 => "1 photo".to_owned(),
        _ => format!("{count} photos"),
    }
}

/// What to tell the user once an export is over.
pub fn export_summary(progress: &ExportProgress, destination: &Path) -> String {
    let folder = photo_name(destination);
    let handled = progress.exported + progress.failed;
    let exported = match handled == progress.total {
        true => format!("{} exported to {folder}", photos(progress.exported)),
        false => format!(
            "Export stopped: {} of {} exported to {folder}",
            progress.exported, progress.total
        ),
    };
    match progress.failed {
        0 => exported,
        failed => format!("{exported}, {} could not be exported", photos(failed)),
    }
}

/// Floats over the bottom of `over`. Returns whether Cancel was asked for.
pub fn export_progress(ui: &egui::Ui, over: egui::Rect, progress: &ExportProgress) -> bool {
    floating_pill(ui, ("export progress", over), |ui| {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(RichText::new(progress_label(progress)).color(color::TEXT));
            ui.add_space(space::S);
            ui.button(STOP_EXPORT_LABEL).clicked()
        })
        .inner
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(exported: usize, failed: usize, total: usize) -> ExportProgress {
        ExportProgress {
            exported,
            failed,
            total,
            is_over: false,
        }
    }

    #[test]
    fn progress_names_the_photo_being_written() {
        assert_eq!(progress_label(&progress(2, 0, 12)), "Exporting 3 of 12");
        assert_eq!(progress_label(&progress(11, 1, 12)), "Exporting 12 of 12");
    }

    #[test]
    fn summary_counts_what_was_exported_and_what_failed() {
        let destination = Path::new("/Users/me/Exports");

        assert_eq!(
            export_summary(&progress(1, 0, 1), destination),
            "1 photo exported to Exports"
        );
        assert_eq!(
            export_summary(&progress(10, 2, 12), destination),
            "10 photos exported to Exports, 2 photos could not be exported"
        );
        assert_eq!(
            export_summary(&progress(3, 0, 12), destination),
            "Export stopped: 3 of 12 exported to Exports"
        );
    }
}
