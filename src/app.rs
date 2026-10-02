use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{Key, KeyboardShortcut, Modifiers};

use crate::design::ui::floating_pill::PILL_STACKING_STEP;
use crate::design::ui::notice::{Notice, notice_toast};
use crate::design::ui::theme::{apply_theme, color, space};
use crate::develop::application::session_edits::{SaveFailure, SessionEdits};
use crate::develop::domain::copied_edit::CopiedEdit;
use crate::develop::domain::development::Development;
use crate::develop::domain::edit::{Edit, FULL_INTENSITY};
use crate::develop::domain::mask::{Mask, MaskShape};
use crate::develop::domain::zone::ZoneMask;
use crate::develop::infrastructure::sidecar_files::SidecarFiles;
use crate::develop::ui::before_badge::before_badge;
use crate::develop::ui::develop_panel::{
    DevelopPanelState, develop_panel, edits_left_alone_warning,
};
use crate::develop::ui::mask_canvas::{MaskCanvasState, PhotoOnScreen, mask_canvas};
use crate::develop::ui::masks_section::MaskSelection;
use crate::engine::infrastructure::engine::Engine;
use crate::engine::infrastructure::source_texture::SourceTexture;
use crate::enhance::application::enhancement_run::EnhancementRun;
use crate::enhance::application::enhancer::{EnhancementError, Enhancer};
use crate::enhance::domain::enhancement_model::TILE_SHAPE;
use crate::enhance::infrastructure::enhanced_upload::upload_with_enhancement;
use crate::enhance::infrastructure::photo_enhancement::enhance_photo;
use crate::enhance::ui::enhancement_progress::{enhancement_label, enhancement_progress};
use crate::export::application::export_run::ExportRun;
use crate::export::domain::export_settings::ExportSettings;
use crate::export::infrastructure::destination_picker::pick_destination;
use crate::export::infrastructure::photo_export::export_photo;
use crate::export::ui::export_dialog::{ExportDialogIntent, export_dialog};
use crate::export::ui::export_progress::{export_progress, export_summary};
use crate::library::domain::session::Session;
use crate::library::infrastructure::photo_files::photo_files_among;
use crate::library::infrastructure::photo_picker::pick_photos_or_folder;
use crate::library::ui::empty_state::empty_state;
use crate::library::ui::filmstrip::{
    FILMSTRIP_HEIGHT, FilmstripIntent, FilmstripPhoto, ThumbnailState, filmstrip,
};
use crate::models::application::model_store::ModelStore;
use crate::models::infrastructure::model_downloads::ModelDownloads;
use crate::models::infrastructure::models_folder::models_folder;
use crate::models::infrastructure::onnx_runner::OnnxRunner;
use crate::photo::application::photo_loader::{Backlog, LoadedPhoto, PhotoLoader};
use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::photo_kind::PhotoKind;
use crate::photo::domain::photo_name::photo_name;
use crate::photo::domain::thumbnail::Thumbnail;
use crate::photo::infrastructure::file_decoder::FileDecoder;
use crate::viewport::domain::view::View;
use crate::viewport::infrastructure::photo_presenter::{
    PhotoPresenter, PresentedPhoto, ShownPhoto,
};
use crate::viewport::ui::photo_status::{photo_failed, photo_loading};
use crate::viewport::ui::photo_viewport::photo_viewport;
use crate::zones::application::detection::{Detected, DetectionAsked};
use crate::zones::application::detection_run::DetectionRun;
use crate::zones::application::zone_detector::{DetectionError, ZoneDetector};
use crate::zones::domain::people_pick::PeoplePick;
use crate::zones::infrastructure::engine_photo_view::EnginePhotoView;
use crate::zones::ui::detection_status::{
    detection_label, detection_status, nothing_found_label, parts_not_found_label,
};
use crate::zones::ui::people_picker::{PeoplePickerIntent, people_picker, person_outlines};

const APP_NAME: &str = "Ziv";
const NO_PHOTO_FOUND_NOTICE: &str = "No photo found in what was opened";
const NO_MODELS_FOLDER_NOTICE: &str = "Models cannot be kept on this Mac: no data folder";
const DEVELOP_PANEL_WIDTH: f32 = 280.0;
const OPEN_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const EXPORT_SETTINGS_KEY: &str = "export settings";
const EXPORT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::E);
const EXPORT_SESSION_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::E);
const BEFORE_KEY: Key = Key::B;
const COPY_EDIT_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::C);
const PASTE_EDIT_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::V);
const UNDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
const REDO_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);

enum ViewedPhoto {
    None,
    Loading(PathBuf),
    Ready {
        photo: Box<PresentedPhoto>,
        kind: PhotoKind,
        view: View,
    },
    Failed {
        path: PathBuf,
        error: DecodeError,
    },
}

/// A detection asked on the viewed photo; viewing another photo abandons it.
struct ZoneDetection {
    asked: DetectionAsked,
    run: DetectionRun<Detected>,
}

/// The enhancement of one photo of the session; it goes on when another photo is viewed.
struct PhotoEnhancement {
    photo: PathBuf,
    run: EnhancementRun,
}

struct UploadedPhoto {
    source: SourceTexture,
    kind: PhotoKind,
}

fn upload_decoded(engine: &Engine, path: &Path) -> Result<UploadedPhoto, DecodeError> {
    let decoded = FileDecoder.decode(path)?;
    Ok(UploadedPhoto {
        source: upload_with_enhancement(engine, path, &decoded.image),
        kind: decoded.kind,
    })
}

pub struct ZivApp {
    egui_context: egui::Context,
    presenter: PhotoPresenter,
    viewed_loader: PhotoLoader<UploadedPhoto>,
    thumbnail_loader: PhotoLoader<Thumbnail>,
    session: Session,
    /// One per photo of the session, in the same order.
    thumbnails: Vec<ThumbnailState>,
    viewed: ViewedPhoto,
    edits: SessionEdits<SidecarFiles>,
    notice: Option<Notice>,
    /// The viewport shows the selected photo without its edit.
    is_before_shown: bool,
    mask_selection: MaskSelection,
    copied_edit: Option<CopiedEdit>,
    engine: Arc<Engine>,
    export_settings: ExportSettings,
    /// The photos the open export dialog is about.
    photos_to_export: Option<Vec<PathBuf>>,
    export_run: Option<ExportRun>,
    zone_detection: Option<ZoneDetection>,
    /// The persons found in the viewed photo, while the user chooses among them.
    people_pick: Option<PeoplePick>,
    enhancement: Option<PhotoEnhancement>,
}

fn detection_failure_notice(error: &DetectionError) -> String {
    format!("Nothing was detected: {error}")
}

fn panel_frame(margin: f32) -> egui::Frame {
    egui::Frame::new().fill(color::PANEL).inner_margin(margin)
}

fn spawn_thumbnail_loader(egui_context: &egui::Context) -> PhotoLoader<Thumbnail> {
    let egui_context = egui_context.clone();
    PhotoLoader::spawn(
        Backlog::LoadAll,
        |path| FileDecoder.decode_thumbnail(path),
        move || egui_context.request_repaint(),
    )
}

impl ZivApp {
    pub fn new(creation: &eframe::CreationContext<'_>) -> Self {
        let render_state = creation
            .wgpu_render_state
            .as_ref()
            .expect("ziv runs on eframe's wgpu backend");
        let engine = Arc::new(Engine::new(
            render_state.device.clone(),
            render_state.queue.clone(),
        ));
        let egui_context = creation.egui_ctx.clone();
        apply_theme(&egui_context);
        let repainting_context = egui_context.clone();
        let uploading_engine = engine.clone();
        let export_settings = creation
            .storage
            .and_then(|storage| eframe::get_value(storage, EXPORT_SETTINGS_KEY))
            .unwrap_or_default();
        Self {
            presenter: PhotoPresenter::new(render_state, engine.clone()),
            engine,
            export_settings,
            photos_to_export: None,
            export_run: None,
            zone_detection: None,
            people_pick: None,
            enhancement: None,
            viewed_loader: PhotoLoader::spawn(
                Backlog::LoadNewestOnly,
                move |path| upload_decoded(&uploading_engine, path),
                move || repainting_context.request_repaint(),
            ),
            thumbnail_loader: spawn_thumbnail_loader(&egui_context),
            egui_context,
            session: Session::default(),
            thumbnails: Vec::new(),
            viewed: ViewedPhoto::None,
            edits: SessionEdits::new(SidecarFiles),
            notice: None,
            is_before_shown: false,
            mask_selection: MaskSelection::default(),
            copied_edit: None,
        }
    }

    /// The app as if the user had just opened `paths` (files or folders).
    pub fn opening(creation: &eframe::CreationContext<'_>, paths: &[PathBuf]) -> Self {
        let mut app = Self::new(creation);
        app.open(paths);
        app
    }

    /// The app as if the user had already chosen where exports go.
    pub fn exporting_to(mut self, destination: PathBuf) -> Self {
        self.export_settings.destination = Some(destination);
        self
    }

    fn open(&mut self, paths: &[PathBuf]) {
        let photos = photo_files_among(paths, |path| FileDecoder.supports(path));
        if self.session.open(photos).is_err() {
            self.show_notice(NO_PHOTO_FOUND_NOTICE);
            return;
        }
        self.notice = None;
        let failure = self.edits.close_session();
        self.report(failure);
        self.request_thumbnails();
        self.view_selected_photo();
    }

    fn request_thumbnails(&mut self) {
        // A new loader: the previous one stops instead of finishing the old session's thumbnails.
        self.thumbnail_loader = spawn_thumbnail_loader(&self.egui_context);
        self.thumbnails = self
            .session
            .photos()
            .iter()
            .map(|path| {
                self.thumbnail_loader.request(path.clone());
                ThumbnailState::Loading
            })
            .collect();
    }

    fn view_selected_photo(&mut self) {
        let Some(path) = self.session.selected_photo() else {
            return;
        };
        self.is_before_shown = false;
        self.mask_selection = self.mask_selection.selecting(None);
        self.zone_detection = None;
        self.people_pick = None;
        self.edits.load(path);
        self.viewed_loader.request(path.to_owned());
        self.viewed = ViewedPhoto::Loading(path.to_owned());
    }

    fn receive_viewed_photo(&mut self) {
        for loaded in self.viewed_loader.take_loaded() {
            let ViewedPhoto::Loading(awaited) = &self.viewed else {
                continue;
            };
            if *awaited != loaded.path {
                continue;
            }
            self.viewed = match loaded.result {
                Ok(uploaded) => ViewedPhoto::Ready {
                    photo: Box::new(self.presenter.present(uploaded.source)),
                    kind: uploaded.kind,
                    view: View::fit(),
                },
                Err(error) => {
                    self.mark_thumbnail_failed(&loaded.path);
                    ViewedPhoto::Failed {
                        path: loaded.path,
                        error,
                    }
                }
            };
        }
    }

    fn mark_thumbnail_failed(&mut self, path: &Path) {
        if let Some(index) = self.session.index_of(path) {
            self.thumbnails[index] = ThumbnailState::Failed;
        }
    }

    fn receive_thumbnails(&mut self) {
        for LoadedPhoto { path, result } in self.thumbnail_loader.take_loaded() {
            let Some(index) = self.session.index_of(&path) else {
                continue;
            };
            self.thumbnails[index] = match result {
                Ok(thumbnail) => ThumbnailState::Ready(self.thumbnail_texture(&path, &thumbnail)),
                Err(_) => ThumbnailState::Failed,
            };
        }
    }

    fn thumbnail_texture(&self, path: &Path, thumbnail: &Thumbnail) -> egui::TextureHandle {
        let size = [thumbnail.width as usize, thumbnail.height as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(size, &thumbnail.rgba);
        self.egui_context
            .load_texture(photo_name(path), image, egui::TextureOptions::LINEAR)
    }

    fn open_dropped_files(&mut self, ui: &egui::Ui) {
        let dropped: Vec<PathBuf> = ui.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_owned())
                .collect()
        });
        if !dropped.is_empty() {
            self.open(&dropped);
        }
    }

    fn show_notice(&mut self, text: impl Into<String>) {
        let now = self.egui_context.input(|input| input.time);
        self.notice = Some(Notice::shown_at(now, text));
    }

    fn report(&mut self, failure: Option<SaveFailure>) {
        if let Some(SaveFailure { photo, reason }) = failure {
            let name = photo_name(&photo);
            self.show_notice(format!("Edits of {name} could not be saved: {reason}"));
        }
    }

    fn save_settled_edits(&mut self, ui: &egui::Ui) {
        let now = ui.input(|input| input.time);
        let failure = self.edits.save_when_due(now);
        self.report(failure);
        if let Some(seconds) = self.edits.seconds_until_save(now) {
            ui.ctx().request_repaint_after_secs(seconds as f32);
        }
    }

    fn notice_over(&mut self, ui: &egui::Ui) {
        let now = ui.input(|input| input.time);
        self.notice.take_if(|notice| notice.is_over_at(now));
        let Some(notice) = &self.notice else {
            return;
        };
        notice_toast(ui, ui.max_rect(), &notice.text);
        ui.ctx()
            .request_repaint_after_secs(notice.seconds_left_at(now) as f32);
    }

    fn open_picked_photos(&mut self) {
        if let Some(picked) = pick_photos_or_folder() {
            self.open(&picked);
        }
    }

    fn apply(&mut self, intent: FilmstripIntent) {
        let previously_selected = self.session.selected_index();
        match intent {
            FilmstripIntent::Select(index) => self.session.select(index),
            FilmstripIntent::SelectPrevious => self.session.select_previous(),
            FilmstripIntent::SelectNext => self.session.select_next(),
            FilmstripIntent::Open => return self.open_picked_photos(),
            FilmstripIntent::Export => return self.ask_to_export_selected_photo(),
        }
        if self.session.selected_index() != previously_selected {
            self.view_selected_photo();
        }
    }

    fn selected_edit(&self) -> Edit {
        self.session
            .selected_photo()
            .map_or_else(Edit::default, |path| self.edits.edit_of(path))
    }

    fn set_selected_edit(&mut self, edit: Edit) {
        let Some(path) = self.session.selected_photo() else {
            return;
        };
        let now = self.egui_context.input(|input| input.time);
        let failure = self.edits.change(path, edit, now);
        self.report(failure);
    }

    /// Undo and redo of the selected photo's edit; a typed value keeps its own.
    fn undo_or_redo(&mut self, ui: &egui::Ui) {
        let Some(path) = self.session.selected_photo() else {
            return;
        };
        let is_pointer_down = ui.input(|input| input.pointer.any_down());
        self.edits.set_gesture_ongoing(path, is_pointer_down);
        if ui.ctx().text_edit_focused() {
            return;
        }
        let now = ui.input(|input| input.time);
        let failure = if ui.input_mut(|input| input.consume_shortcut(&REDO_SHORTCUT)) {
            self.edits.redo(path, now)
        } else if ui.input_mut(|input| input.consume_shortcut(&UNDO_SHORTCUT)) {
            self.edits.undo(path, now)
        } else {
            None
        };
        self.report(failure);
    }

    fn toggle_before_on_its_key(&mut self, ui: &egui::Ui) {
        let is_typing = ui.ctx().text_edit_focused();
        if !is_typing && ui.input_mut(|input| input.consume_key(Modifiers::NONE, BEFORE_KEY)) {
            self.is_before_shown = !self.is_before_shown;
        }
    }

    fn copy_or_paste_edit(&mut self, ui: &egui::Ui) {
        let ViewedPhoto::Ready { kind, .. } = &self.viewed else {
            return;
        };
        let kind = *kind;
        if ui.input_mut(|input| input.consume_shortcut(&COPY_EDIT_SHORTCUT)) {
            self.copied_edit = Some(CopiedEdit::of(self.selected_edit(), &kind));
        }
        if !ui.input_mut(|input| input.consume_shortcut(&PASTE_EDIT_SHORTCUT)) {
            return;
        }
        if let Some(copied) = &self.copied_edit {
            let pasted = copied.pasted_onto(&self.selected_edit(), &kind);
            self.is_before_shown = false;
            self.set_selected_edit(pasted);
        }
    }

    fn ask_to_export_selected_photo(&mut self) {
        let selected = self.session.selected_photo().map(Path::to_owned);
        self.ask_to_export(selected.into_iter().collect());
    }

    fn ask_to_export(&mut self, photos: Vec<PathBuf>) {
        if photos.is_empty() || self.export_run.is_some() {
            return;
        }
        self.photos_to_export = Some(photos);
    }

    fn ask_to_export_on_shortcut(&mut self, ui: &egui::Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&EXPORT_SESSION_SHORTCUT)) {
            self.ask_to_export(self.session.photos().to_vec());
        }
        if ui.input_mut(|input| input.consume_shortcut(&EXPORT_SHORTCUT)) {
            self.ask_to_export_selected_photo();
        }
    }

    fn export_dialog_window(&mut self, ui: &egui::Ui) {
        let Some(photos) = &self.photos_to_export else {
            return;
        };
        let modal = egui::Modal::new(egui::Id::new("export dialog"))
            .frame(egui::Frame::window(ui.style()).inner_margin(space::L))
            .show(ui.ctx(), |ui| {
                export_dialog(ui, photos, &self.export_settings)
            });
        let is_dismissed = modal.should_close();
        let output = modal.inner;
        self.export_settings = output.settings;
        match output.intent {
            Some(ExportDialogIntent::ChooseDestination) => {
                if let Some(destination) = pick_destination() {
                    self.export_settings.destination = Some(destination);
                }
            }
            Some(ExportDialogIntent::Export) => self.start_export(),
            Some(ExportDialogIntent::Cancel) => self.photos_to_export = None,
            None if is_dismissed => self.photos_to_export = None,
            None => {}
        }
    }

    fn start_export(&mut self) {
        let Some(photos) = self.photos_to_export.take() else {
            return;
        };
        let failure = self.edits.save_now();
        self.report(failure);
        let engine = self.engine.clone();
        let settings = self.export_settings.clone();
        let repainting_context = self.egui_context.clone();
        self.export_run = Some(ExportRun::start(
            photos,
            move |photo| export_photo(&engine, photo, &settings),
            move || repainting_context.request_repaint(),
        ));
    }

    /// Progress while an export runs, then its result as a notice.
    fn export_progress_over(&mut self, ui: &egui::Ui) {
        let Some(run) = &self.export_run else {
            return;
        };
        let progress = run.progress();
        if !progress.is_over {
            if export_progress(ui, ui.max_rect(), &progress) {
                run.cancel();
            }
            return;
        }
        self.export_run = None;
        if let Some(destination) = self.export_settings.destination.clone() {
            self.show_notice(export_summary(&progress, &destination));
        }
    }

    fn can_masks_be_drawn(&self) -> bool {
        !self.is_before_shown
            && self.people_pick.is_none()
            && self.photos_to_export.is_none()
            && self.edits_left_alone_reason().is_none()
    }

    fn draw_masks(&mut self, ui: &mut egui::Ui, photo: &PhotoOnScreen) {
        let edit = self.selected_edit();
        let shown = MaskCanvasState {
            masks: edit.masks.clone(),
            selection: self.mask_selection,
        };
        let left = mask_canvas(ui, photo, shown);
        self.mask_selection = left.selection;
        self.set_selected_edit(Edit {
            masks: left.masks,
            ..edit
        });
    }

    fn start_detection(&mut self, asked: DetectionAsked) {
        let ViewedPhoto::Ready { photo, kind, .. } = &self.viewed else {
            return;
        };
        let Some(folder) = models_folder() else {
            return self.show_notice(NO_MODELS_FOLDER_NOTICE);
        };
        let view = EnginePhotoView {
            engine: self.engine.clone(),
            source: photo.source().clone(),
            kind: *kind,
        };
        let repainting_context = self.egui_context.clone();
        let detected = asked.clone();
        let run = DetectionRun::start(
            move |on_step| {
                let store = ModelStore::new(folder, ModelDownloads);
                ZoneDetector::new(store, OnnxRunner::default()).detected(&detected, &view, on_step)
            },
            move || repainting_context.request_repaint(),
        );
        self.zone_detection = Some(ZoneDetection { asked, run });
    }

    fn receive_detection(&mut self) {
        let Some(result) = self
            .zone_detection
            .as_ref()
            .and_then(|detection| detection.run.take_result())
        else {
            return;
        };
        let Some(ZoneDetection { asked, .. }) = self.zone_detection.take() else {
            return;
        };
        match result {
            Ok(Detected::Persons(persons)) if !persons.is_empty() => {
                self.people_pick = Some(PeoplePick::among(persons));
            }
            Ok(Detected::Masks(masks)) if !masks.is_empty() => {
                let missing = asked.parts_missing_from(&masks);
                self.add_detected_masks(masks);
                if !missing.is_empty() {
                    self.show_notice(parts_not_found_label(&missing));
                }
            }
            Ok(_) => self.show_notice(nothing_found_label(&asked)),
            Err(error) => self.show_notice(detection_failure_notice(&error)),
        }
    }

    /// As many of `masks` as the photo has room for, the last one selected.
    fn add_detected_masks(&mut self, masks: Vec<ZoneMask>) {
        let mut edit = self.selected_edit();
        for mask in masks {
            if edit.has_room_for_a_mask() {
                edit.masks.push(Mask::of(MaskShape::Zone(mask)));
            }
        }
        self.is_before_shown = false;
        self.mask_selection = self
            .mask_selection
            .selecting(edit.masks.len().checked_sub(1));
        self.set_selected_edit(edit);
    }

    fn detection_status_over(&self, ui: &egui::Ui) {
        if let Some(detection) = &self.zone_detection {
            let label = detection_label(&detection.asked, detection.run.step());
            detection_status(ui, ui.max_rect(), &label);
        }
    }

    /// The persons outlined on the photo and the picker to choose among them.
    fn pick_people(&mut self, ui: &egui::Ui, photo: &PhotoOnScreen) {
        let Some(pick) = &self.people_pick else {
            return;
        };
        person_outlines(ui, (photo.area, photo.whole_photo), pick);
        let left = people_picker(ui, photo.area, pick);
        self.people_pick = Some(left.pick);
        match left.intent {
            Some(PeoplePickerIntent::Create) => self.mask_picked_people(),
            Some(PeoplePickerIntent::Cancel) => self.people_pick = None,
            None => {}
        }
    }

    fn mask_picked_people(&mut self) {
        let Some(pick) = self.people_pick.take() else {
            return;
        };
        self.start_detection(DetectionAsked::PersonParts {
            persons: pick.chosen_persons(),
            parts: pick.chosen_parts(),
        });
    }

    fn is_viewed_photo_enhanced(&self) -> bool {
        matches!(&self.viewed, ViewedPhoto::Ready { photo, .. } if photo.source().is_enhanced())
    }

    /// Enhances the viewed photo on its own thread. Its result goes straight
    /// onto the photo on screen when that photo is still the one viewed.
    fn start_enhancement(&mut self) {
        let (ViewedPhoto::Ready { photo: viewed, .. }, Some(photo)) =
            (&self.viewed, self.session.selected_photo())
        else {
            return;
        };
        if self.enhancement.is_some() {
            return;
        }
        let Some(folder) = models_folder() else {
            return self.show_notice(NO_MODELS_FOLDER_NOTICE);
        };
        let photo = photo.to_owned();
        let enhanced_photo = photo.clone();
        let on_screen = Arc::downgrade(viewed.source());
        let engine = self.engine.clone();
        let repainting_context = self.egui_context.clone();
        let run = EnhancementRun::start(
            move |on_step| {
                let store = ModelStore::new(folder, ModelDownloads);
                let enhancer = Enhancer::new(store, OnnxRunner::on_gpu(&TILE_SHAPE));
                let enhanced = enhance_photo(&enhancer, &enhanced_photo, on_step)?;
                if let Some(source) = on_screen.upgrade() {
                    engine.upload_enhancement(&source, &enhanced);
                }
                Ok(())
            },
            move || repainting_context.request_repaint(),
        );
        self.enhancement = Some(PhotoEnhancement { photo, run });
    }

    fn receive_enhancement(&mut self) {
        let Some(result) = self
            .enhancement
            .as_ref()
            .and_then(|enhancement| enhancement.run.take_result())
        else {
            return;
        };
        let Some(PhotoEnhancement { photo, .. }) = self.enhancement.take() else {
            return;
        };
        match result {
            Ok(()) => self.show_at_full_intensity(&photo),
            Err(EnhancementError::Cancelled) => {}
            Err(error) => {
                let name = photo_name(&photo);
                self.show_notice(format!("{name} could not be enhanced: {error}"));
            }
        }
    }

    fn show_at_full_intensity(&mut self, photo: &Path) {
        self.edits.load(photo);
        let enhanced = Edit {
            enhancement_intensity: FULL_INTENSITY,
            ..self.edits.edit_of(photo)
        };
        let now = self.egui_context.input(|input| input.time);
        let failure = self.edits.change(photo, enhanced, now);
        self.report(failure);
        // Viewed again since Enhance was asked: what is on screen was loaded before the file was there.
        let is_selected = self.session.selected_photo() == Some(photo);
        if is_selected && !self.is_viewed_photo_enhanced() {
            self.view_selected_photo();
        }
    }

    /// Above the place of the other messages when one of them is shown.
    fn enhancement_progress_over(&self, ui: &egui::Ui) {
        let Some(enhancement) = &self.enhancement else {
            return;
        };
        let is_place_taken = self.export_run.is_some() || self.zone_detection.is_some();
        let over = match is_place_taken {
            true => ui
                .max_rect()
                .translate(egui::vec2(0.0, -PILL_STACKING_STEP)),
            false => ui.max_rect(),
        };
        let label = enhancement_label(&enhancement.photo, enhancement.run.step());
        if enhancement_progress(ui, over, &label) {
            enhancement.run.cancel();
        }
    }

    fn edits_left_alone_reason(&self) -> Option<&str> {
        self.edits
            .unusable_storage_of(self.session.selected_photo()?)
    }

    /// Shows the panel, disabled until the photo is ready, and returns how
    /// that photo is to be shown.
    fn develop_side_panel(&mut self, ui: &mut egui::Ui) -> ShownPhoto {
        if self.session.photos().is_empty() {
            return ShownPhoto::default();
        }
        let ready_kind = match &self.viewed {
            ViewedPhoto::Ready { kind, .. } => Some(*kind),
            _ => None,
        };
        let kind = ready_kind.unwrap_or(PhotoKind::StandardImage);
        let selected = self.selected_edit();
        let left = egui::Panel::right("develop")
            .exact_size(DEVELOP_PANEL_WIDTH)
            .frame(panel_frame(space::L))
            .show(ui, |ui| {
                let left_alone = self.edits_left_alone_reason();
                if ready_kind.is_none() || left_alone.is_some() {
                    ui.disable();
                }
                let shown = DevelopPanelState {
                    edit: selected,
                    mask_selection: self.mask_selection,
                    is_before_shown: self.is_before_shown,
                    is_detecting: self.zone_detection.is_some() || self.people_pick.is_some(),
                    asked_detection: None,
                    is_enhanced: self.is_viewed_photo_enhanced(),
                    is_enhancing: self.enhancement.is_some(),
                    is_enhancement_asked: false,
                };
                let left = develop_panel(ui, &kind, shown);
                if let Some(reason) = left_alone {
                    edits_left_alone_warning(ui, reason);
                }
                left
            })
            .inner;
        if ready_kind.is_some() {
            self.set_selected_edit(left.edit.clone());
            self.mask_selection = left.mask_selection;
            self.is_before_shown = left.is_before_shown;
        }
        if let Some(tool) = left.asked_detection {
            self.start_detection(tool.into());
        }
        if left.is_enhancement_asked {
            self.start_enhancement();
        }
        if self.is_before_shown {
            let development = Development {
                kind,
                edit: Edit::default(),
            };
            return ShownPhoto {
                development,
                overlaid_mask: None,
            };
        }
        ShownPhoto {
            overlaid_mask: left.mask_selection.overlaid_mask(&left.edit).cloned(),
            development: Development {
                kind,
                edit: left.edit,
            },
        }
    }

    fn filmstrip_panel(&self, ui: &mut egui::Ui) -> Option<FilmstripIntent> {
        let photos: Vec<FilmstripPhoto<'_>> = self
            .session
            .photos()
            .iter()
            .zip(&self.thumbnails)
            .map(|(path, thumbnail)| FilmstripPhoto { path, thumbnail })
            .collect();
        egui::Panel::bottom("filmstrip")
            .exact_size(FILMSTRIP_HEIGHT)
            .frame(panel_frame(space::M))
            .show(ui, |ui| {
                filmstrip(ui, &photos, self.session.selected_index())
            })
            .inner
    }
}

impl eframe::App for ZivApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.receive_viewed_photo();
        self.receive_thumbnails();
        self.receive_detection();
        self.receive_enhancement();
        self.open_dropped_files(ui);
        let is_dialog_open = self.photos_to_export.is_some();
        if !is_dialog_open {
            self.undo_or_redo(ui);
            self.toggle_before_on_its_key(ui);
            self.copy_or_paste_edit(ui);
            self.ask_to_export_on_shortcut(ui);
        }
        self.save_settled_edits(ui);
        let mut open_requested = ui.input_mut(|input| input.consume_shortcut(&OPEN_SHORTCUT));

        let filmstrip_intent = if self.session.photos().is_empty() {
            None
        } else {
            self.filmstrip_panel(ui)
        };
        let shown = self.develop_side_panel(ui);
        let mut masks_on_screen = None;
        let canvas = egui::Frame::new().fill(color::CANVAS);
        egui::CentralPanel::default().frame(canvas).show(ui, |ui| {
            self.notice_over(ui);
            self.export_progress_over(ui);
            self.detection_status_over(ui);
            self.enhancement_progress_over(ui);
            match &mut self.viewed {
                ViewedPhoto::None => open_requested |= empty_state(ui),
                ViewedPhoto::Loading(path) => photo_loading(ui, path),
                ViewedPhoto::Ready { photo, view, .. } => {
                    let output = photo_viewport(ui, photo.size(), *view, |placement| {
                        self.presenter.render_at(photo, placement, &shown)
                    });
                    *view = output.view;
                    masks_on_screen = Some(PhotoOnScreen {
                        area: ui.max_rect(),
                        whole_photo: output.whole_photo_rect,
                    });
                    if self.is_before_shown {
                        before_badge(ui, ui.max_rect());
                    }
                }
                ViewedPhoto::Failed { path, error } => photo_failed(ui, path, error),
            }
            if let Some(photo) = masks_on_screen.filter(|_| self.can_masks_be_drawn()) {
                self.draw_masks(ui, &photo);
            }
            if let Some(photo) = &masks_on_screen {
                self.pick_people(ui, photo);
            }
        });

        self.export_dialog_window(ui);
        if is_dialog_open {
            return;
        }
        if let Some(intent) = filmstrip_intent {
            self.apply(intent);
        }
        if open_requested {
            self.open_picked_photos();
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, EXPORT_SETTINGS_KEY, &self.export_settings);
    }

    fn on_exit(&mut self) {
        let _ = self.edits.save_now();
    }
}

pub fn run() -> eframe::Result<()> {
    eframe::run_native(
        APP_NAME,
        eframe::NativeOptions::default(),
        Box::new(|creation| Ok(Box::new(ZivApp::new(creation)))),
    )
}
