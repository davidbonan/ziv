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
use crate::develop::infrastructure::sidecar_files::{SidecarFiles, has_sidecar};
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
use crate::histogram::infrastructure::developed_histogram::DevelopedHistogram;
use crate::histogram::ui::histogram_plot::histogram_plot;
use crate::library::application::stored_catalog::StoredCatalog;
use crate::library::domain::catalog::Catalog;
use crate::library::domain::series::Series;
use crate::library::infrastructure::catalog_file::CatalogFile;
use crate::library::infrastructure::photo_files::import_of;
use crate::library::infrastructure::photo_picker::{
    pick_photos_or_folder, pick_series_folder, show_in_finder,
};
use crate::library::infrastructure::series_covers::SeriesCovers;
use crate::library::infrastructure::thumbnail_texture::thumbnail_texture;
use crate::library::infrastructure::today::today;
use crate::library::ui::empty_state::empty_state;
use crate::library::ui::filmstrip::{
    FILMSTRIP_HEIGHT, FilmstripIntent, FilmstripPhoto, ThumbnailState, filmstrip,
};
use crate::library::ui::series_sidebar::{
    SERIES_SIDEBAR_WIDTH, SeriesRow, SeriesShown, SidebarIntent, series_sidebar,
    unusable_catalog_warning,
};
use crate::models::application::model_store::ModelStore;
use crate::models::infrastructure::model_downloads::ModelDownloads;
use crate::models::infrastructure::models_folder::models_folder;
use crate::models::infrastructure::onnx_runner::OnnxRunner;
use crate::photo::application::photo_loader::{Backlog, LoadedPhoto, PhotoLoader};
use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::photo_kind::PhotoKind;
use crate::photo::domain::photo_name::photo_name;
use crate::photo::domain::shooting_data::ShootingData;
use crate::photo::domain::thumbnail::Thumbnail;
use crate::photo::infrastructure::file_decoder::FileDecoder;
use crate::photo::infrastructure::thumbnail_cache::ThumbnailCache;
use crate::photo::ui::shooting_data_line::shooting_data_line;
use crate::shell::ui::top_bar::{TOP_BAR_HEIGHT, TopBarIntent, TopBarShown, top_bar};
use crate::viewport::domain::view::View;
use crate::viewport::domain::zoom_readout::zoom_readout;
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
const NOTHING_LOCATED_NOTICE: &str = "None of the photos of this series is in that folder";
const PHOTO_NOT_FOUND_REASON: &str = "not found";
const NO_MODELS_FOLDER_NOTICE: &str = "Models cannot be kept on this Mac: no data folder";
const DEVELOP_PANEL_WIDTH: f32 = 300.0;
const OPEN_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const EXPORT_SETTINGS_KEY: &str = "export settings";
const SIDEBAR_SHOWN_KEY: &str = "series sidebar shown";
const SIDEBAR_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::L);
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
        shooting_data: ShootingData,
        histogram: DevelopedHistogram,
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
    shooting_data: ShootingData,
}

fn upload_decoded(engine: &Engine, path: &Path) -> Result<UploadedPhoto, DecodeError> {
    let decoded = FileDecoder.decode(path)?;
    Ok(UploadedPhoto {
        source: upload_with_enhancement(engine, path, &decoded.image),
        kind: decoded.kind,
        shooting_data: decoded.shooting_data,
    })
}

pub struct ZivApp {
    egui_context: egui::Context,
    presenter: PhotoPresenter,
    viewed_loader: PhotoLoader<UploadedPhoto>,
    thumbnail_loader: PhotoLoader<Thumbnail>,
    thumbnail_cache: ThumbnailCache,
    covers: SeriesCovers,
    catalog: StoredCatalog<CatalogFile>,
    /// Why the series of this run will be forgotten, when what is stored cannot be used.
    catalog_warning: Option<String>,
    /// One per photo of the open series, in the same order.
    thumbnails: Vec<ThumbnailState>,
    is_sidebar_shown: bool,
    /// One per series of the catalog, in the same order.
    series_on_disk: Vec<SeriesOnDisk>,
    /// Whether each photo of the open series is edited, in the same order.
    edited_marks: Vec<bool>,
    /// The zoom the viewport showed the photo at, at the last frame.
    zoom_readout: Option<String>,
    /// An edit was waiting to be saved at the last frame.
    was_edit_unsaved: bool,
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

/// What the files of a series say about it.
struct SeriesOnDisk {
    edited_count: usize,
    is_missing: bool,
}

impl SeriesOnDisk {
    fn of(series: &Series) -> Self {
        Self {
            edited_count: series.edited_count(has_sidecar),
            is_missing: series.is_missing(Path::exists),
        }
    }
}

fn detection_failure_notice(error: &DetectionError) -> String {
    format!("Nothing was detected: {error}")
}

fn panel_frame(margin: f32) -> egui::Frame {
    egui::Frame::new().fill(color::PANEL).inner_margin(margin)
}

fn thumbnail_through(cache: &ThumbnailCache, photo: &Path) -> Result<Thumbnail, DecodeError> {
    cache.thumbnail_of(photo, |photo| FileDecoder.decode_thumbnail(photo))
}

fn spawn_thumbnail_loader(
    egui_context: &egui::Context,
    cache: &ThumbnailCache,
) -> PhotoLoader<Thumbnail> {
    let egui_context = egui_context.clone();
    let cache = cache.clone();
    PhotoLoader::spawn(
        Backlog::LoadAll,
        move |photo| thumbnail_through(&cache, photo),
        move || egui_context.request_repaint(),
    )
}

impl ZivApp {
    pub fn new(creation: &eframe::CreationContext<'_>) -> Self {
        let catalog_file = CatalogFile::of_this_mac();
        Self::keeping(creation, catalog_file, ThumbnailCache::of_this_mac())
    }

    /// The app keeping what it remembers in `folder` instead of this Mac's data folder.
    pub fn keeping_data_in(creation: &eframe::CreationContext<'_>, folder: PathBuf) -> Self {
        let thumbnail_cache = ThumbnailCache::in_folder(folder.join("thumbnails"));
        Self::keeping(creation, CatalogFile::in_folder(folder), thumbnail_cache)
    }

    fn keeping(
        creation: &eframe::CreationContext<'_>,
        catalog_file: CatalogFile,
        thumbnail_cache: ThumbnailCache,
    ) -> Self {
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
        let is_sidebar_shown = creation
            .storage
            .and_then(|storage| eframe::get_value(storage, SIDEBAR_SHOWN_KEY))
            .unwrap_or(true);
        let catalog_path = catalog_file.path();
        let catalog = StoredCatalog::read_from(catalog_file);
        let mut app = Self {
            is_sidebar_shown,
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
            thumbnail_loader: spawn_thumbnail_loader(&egui_context, &thumbnail_cache),
            covers: SeriesCovers::spawn(&egui_context, {
                let cache = thumbnail_cache.clone();
                move |photo| thumbnail_through(&cache, photo)
            }),
            thumbnail_cache,
            egui_context,
            catalog_warning: catalog
                .unusable_storage()
                .map(|reason| unusable_catalog_warning(catalog_path.as_deref(), reason)),
            catalog,
            thumbnails: Vec::new(),
            series_on_disk: Vec::new(),
            edited_marks: Vec::new(),
            zoom_readout: None,
            was_edit_unsaved: false,
            viewed: ViewedPhoto::None,
            edits: SessionEdits::new(SidecarFiles),
            notice: None,
            is_before_shown: false,
            mask_selection: MaskSelection::default(),
            copied_edit: None,
        };
        app.show_open_series();
        app
    }

    /// The app as if the user had then opened `paths` (files or folders).
    pub fn opening(mut self, paths: &[PathBuf]) -> Self {
        self.open(paths);
        self
    }

    /// The app as if the user had already chosen where exports go.
    pub fn exporting_to(mut self, destination: PathBuf) -> Self {
        self.export_settings.destination = Some(destination);
        self
    }

    fn open(&mut self, paths: &[PathBuf]) {
        let import = import_of(paths, |path| FileDecoder.supports(path));
        let Some(series) = Series::imported(import, today()) else {
            return self.show_notice(NO_PHOTO_FOUND_NOTICE);
        };
        self.notice = None;
        self.change_catalog(|catalog| catalog.import(series));
        self.show_open_series();
    }

    fn change_catalog(&mut self, change: impl FnOnce(&mut Catalog)) {
        if let Some(reason) = self.catalog.change(change) {
            self.show_notice(format!("The catalog could not be saved: {reason}"));
        }
    }

    fn show_open_series(&mut self) {
        let failure = self.edits.close_session();
        self.report(failure);
        self.look_at_series_on_disk();
        self.request_thumbnails();
        self.view_selected_photo();
    }

    fn look_at_series_on_disk(&mut self) {
        let series = self.catalog.current().series().iter();
        self.series_on_disk = series.map(SeriesOnDisk::of).collect();
        let photos = self.catalog.current().photos().iter();
        self.edited_marks = photos.map(|photo| has_sidecar(photo)).collect();
        self.covers.request_those_of(self.catalog.current());
    }

    fn request_thumbnails(&mut self) {
        // A new loader: the previous one stops instead of finishing the old session's thumbnails.
        self.thumbnail_loader = spawn_thumbnail_loader(&self.egui_context, &self.thumbnail_cache);
        let photos = self.catalog.current().photos();
        let requested = photos.iter().map(|photo| {
            if !photo.exists() {
                return ThumbnailState::NotFound;
            }
            self.thumbnail_loader.request(photo.clone());
            ThumbnailState::Loading
        });
        self.thumbnails = requested.collect();
    }

    fn view_selected_photo(&mut self) {
        let Some(path) = self.catalog.current().selected_photo() else {
            self.viewed = ViewedPhoto::None;
            return;
        };
        self.is_before_shown = false;
        self.mask_selection = self.mask_selection.selecting(None);
        self.zone_detection = None;
        self.people_pick = None;
        self.edits.load(path);
        if !path.exists() {
            self.viewed = ViewedPhoto::Failed {
                path: path.to_owned(),
                error: DecodeError::new(PHOTO_NOT_FOUND_REASON),
            };
            return;
        }
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
                    shooting_data: uploaded.shooting_data,
                    histogram: DevelopedHistogram::default(),
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
        if let Some(index) = self.catalog.current().index_of(path) {
            self.thumbnails[index] = ThumbnailState::Failed;
        }
    }

    fn receive_thumbnails(&mut self) {
        for LoadedPhoto { path, result } in self.thumbnail_loader.take_loaded() {
            let Some(index) = self.catalog.current().index_of(&path) else {
                continue;
            };
            self.thumbnails[index] = match result {
                Ok(thumbnail) => {
                    ThumbnailState::Ready(thumbnail_texture(&self.egui_context, &path, &thumbnail))
                }
                Err(_) => ThumbnailState::Failed,
            };
        }
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
        let until_save = self.edits.seconds_until_save(now);
        if let Some(seconds) = until_save {
            ui.ctx().request_repaint_after_secs(seconds as f32);
        }
        if self.was_edit_unsaved && until_save.is_none() {
            self.look_at_series_on_disk();
        }
        self.was_edit_unsaved = until_save.is_some();
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
        let previously_selected = self.catalog.current().selected_index();
        match intent {
            FilmstripIntent::Select(index) => self.change_catalog(|catalog| catalog.select(index)),
            FilmstripIntent::SelectPrevious => self.change_catalog(Catalog::select_previous),
            FilmstripIntent::SelectNext => self.change_catalog(Catalog::select_next),
        }
        if self.catalog.current().selected_index() != previously_selected {
            self.view_selected_photo();
        }
    }

    fn apply_to_window(&mut self, intent: TopBarIntent) {
        match intent {
            TopBarIntent::ToggleSidebar => self.is_sidebar_shown = !self.is_sidebar_shown,
            TopBarIntent::ToggleBefore => self.is_before_shown = !self.is_before_shown,
            TopBarIntent::Export => self.ask_to_export_selected_photo(),
        }
    }

    fn apply_to_series(&mut self, intent: SidebarIntent) {
        match intent {
            SidebarIntent::Import => self.open_picked_photos(),
            SidebarIntent::Open(index) => self.open_series(index),
            SidebarIntent::Rename { index, name } => {
                self.change_catalog(|catalog| catalog.rename(index, &name));
            }
            SidebarIntent::ShowInFinder(index) => self.show_series_in_finder(index),
            SidebarIntent::Locate(index) => {
                if let Some(folder) = pick_series_folder() {
                    self.relocate_series(index, &folder);
                }
            }
            SidebarIntent::Remove(index) => self.remove_series(index),
        }
    }

    fn show_series_in_finder(&mut self, index: usize) {
        let series = self.catalog.current().series().get(index);
        let Some(folder) = series.and_then(Series::folder_to_show) else {
            return;
        };
        if let Err(reason) = show_in_finder(folder) {
            self.show_notice(format!("The Finder could not be opened: {reason}"));
        }
    }

    fn relocate_series(&mut self, index: usize, folder: &Path) {
        let mut found = 0;
        self.change_catalog(|catalog| found = catalog.relocate(index, folder, Path::exists));
        if found == 0 {
            return self.show_notice(NOTHING_LOCATED_NOTICE);
        }
        match self.catalog.current().open_index() == Some(index) {
            true => self.show_open_series(),
            false => self.look_at_series_on_disk(),
        }
    }

    fn remove_series(&mut self, index: usize) {
        let was_open = self.catalog.current().open_index() == Some(index);
        self.change_catalog(|catalog| catalog.remove(index));
        match was_open {
            true => self.show_open_series(),
            false => self.look_at_series_on_disk(),
        }
    }

    fn open_series(&mut self, index: usize) {
        let previously_open = self.catalog.current().open_index();
        self.change_catalog(|catalog| catalog.open(index));
        if self.catalog.current().open_index() != previously_open {
            self.show_open_series();
        }
    }

    fn selected_edit(&self) -> Edit {
        self.catalog
            .current()
            .selected_photo()
            .map_or_else(Edit::default, |path| self.edits.edit_of(path))
    }

    fn set_selected_edit(&mut self, edit: Edit) {
        let Some(path) = self.catalog.current().selected_photo() else {
            return;
        };
        let now = self.egui_context.input(|input| input.time);
        let failure = self.edits.change(path, edit, now);
        self.report(failure);
    }

    /// Undo and redo of the selected photo's edit; a typed value keeps its own.
    fn undo_or_redo(&mut self, ui: &egui::Ui) {
        let Some(path) = self.catalog.current().selected_photo() else {
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

    fn copy_or_paste_edit_on_shortcut(&mut self, ui: &egui::Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&COPY_EDIT_SHORTCUT)) {
            self.copy_edit();
        }
        if ui.input_mut(|input| input.consume_shortcut(&PASTE_EDIT_SHORTCUT)) {
            self.paste_edit();
        }
    }

    fn copy_edit(&mut self) {
        if let ViewedPhoto::Ready { kind, .. } = &self.viewed {
            self.copied_edit = Some(CopiedEdit::of(self.selected_edit(), kind));
        }
    }

    fn paste_edit(&mut self) {
        let (ViewedPhoto::Ready { kind, .. }, Some(copied)) = (&self.viewed, &self.copied_edit)
        else {
            return;
        };
        let pasted = copied.pasted_onto(&self.selected_edit(), kind);
        self.is_before_shown = false;
        self.set_selected_edit(pasted);
    }

    fn ask_to_export_selected_photo(&mut self) {
        let selected = self.catalog.current().selected_photo().map(Path::to_owned);
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
            self.ask_to_export(self.catalog.current().photos().to_vec());
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
            (&self.viewed, self.catalog.current().selected_photo())
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
        let is_selected = self.catalog.current().selected_photo() == Some(photo);
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
            .unusable_storage_of(self.catalog.current().selected_photo()?)
    }

    /// Shows the panel, disabled until the photo is ready, and returns how
    /// that photo is to be shown.
    fn develop_side_panel(&mut self, ui: &mut egui::Ui) -> ShownPhoto {
        if self.catalog.current().photos().is_empty() {
            return ShownPhoto::default();
        }
        let ready_kind = match &self.viewed {
            ViewedPhoto::Ready { kind, .. } => Some(*kind),
            _ => None,
        };
        let (histogram, shooting_data) = match &self.viewed {
            ViewedPhoto::Ready {
                histogram,
                shooting_data,
                ..
            } => (histogram.latest(), *shooting_data),
            _ => (None, ShootingData::default()),
        };
        let kind = ready_kind.unwrap_or(PhotoKind::StandardImage);
        let selected = self.selected_edit();
        let left = egui::Panel::right("develop")
            .exact_size(DEVELOP_PANEL_WIDTH)
            .frame(panel_frame(space::L))
            .show(ui, |ui| {
                histogram_plot(ui, histogram);
                shooting_data_line(ui, &shooting_data);
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
                    can_paste: self.copied_edit.is_some(),
                    is_copy_asked: false,
                    is_paste_asked: false,
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
        if left.is_copy_asked {
            self.copy_edit();
        }
        if left.is_paste_asked {
            self.paste_edit();
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

    /// Counts the histogram of the photo as it is shown; the panel draws it at the next frame.
    fn follow_histogram(&mut self, shown: &ShownPhoto) {
        let ViewedPhoto::Ready {
            photo, histogram, ..
        } = &mut self.viewed
        else {
            return;
        };
        if histogram.follow(&self.engine, photo.source(), &shown.development) {
            self.egui_context.request_repaint();
        }
    }

    fn series_side_panel(&self, ui: &mut egui::Ui) -> Option<SidebarIntent> {
        let catalog = self.catalog.current();
        let rows: Vec<SeriesRow<'_>> = catalog
            .series()
            .iter()
            .zip(&self.series_on_disk)
            .map(|(series, on_disk)| SeriesRow {
                name: &series.name,
                photo_count: series.session.photos().len(),
                edited_count: on_disk.edited_count,
                is_missing: on_disk.is_missing,
                imported_on: series.imported_on,
                cover: self.covers.cover_of(series),
            })
            .collect();
        egui::Panel::left("series")
            .exact_size(SERIES_SIDEBAR_WIDTH)
            .frame(panel_frame(space::S))
            .show(ui, |ui| {
                let shown = SeriesShown {
                    rows: &rows,
                    open: catalog.open_index(),
                    warning: self.catalog_warning.as_deref(),
                };
                series_sidebar(ui, &shown)
            })
            .inner
    }

    fn top_bar_panel(&self, ui: &mut egui::Ui) -> Option<TopBarIntent> {
        let catalog = self.catalog.current();
        let photo_name = catalog.selected_photo().map(photo_name);
        let shown = TopBarShown {
            is_sidebar_shown: self.is_sidebar_shown,
            series_name: catalog.open_series().map(|series| series.name.as_str()),
            photo_name: photo_name.as_deref(),
            zoom_readout: self.zoom_readout.as_deref(),
            is_before_shown: self.is_before_shown,
            can_before_be_shown: matches!(self.viewed, ViewedPhoto::Ready { .. }),
            can_export: catalog.selected_photo().is_some(),
        };
        egui::Panel::top("top bar")
            .exact_size(TOP_BAR_HEIGHT)
            .frame(panel_frame(space::S))
            .show(ui, |ui| top_bar(ui, &shown))
            .inner
    }

    fn filmstrip_panel(&self, ui: &mut egui::Ui) -> Option<FilmstripIntent> {
        let photos: Vec<FilmstripPhoto<'_>> = self
            .catalog
            .current()
            .photos()
            .iter()
            .zip(&self.thumbnails)
            .zip(&self.edited_marks)
            .map(|((path, thumbnail), is_edited)| FilmstripPhoto {
                path,
                thumbnail,
                is_edited: *is_edited,
            })
            .collect();
        egui::Panel::bottom("filmstrip")
            .exact_size(FILMSTRIP_HEIGHT)
            .frame(panel_frame(space::M))
            .show(ui, |ui| {
                filmstrip(ui, &photos, self.catalog.current().selected_index())
            })
            .inner
    }
}

impl eframe::App for ZivApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.receive_viewed_photo();
        self.receive_thumbnails();
        self.covers.receive_loaded();
        self.receive_detection();
        self.receive_enhancement();
        self.open_dropped_files(ui);
        let is_dialog_open = self.photos_to_export.is_some();
        if !is_dialog_open {
            self.undo_or_redo(ui);
            self.toggle_before_on_its_key(ui);
            self.copy_or_paste_edit_on_shortcut(ui);
            self.ask_to_export_on_shortcut(ui);
        }
        self.save_settled_edits(ui);
        let mut open_requested = ui.input_mut(|input| input.consume_shortcut(&OPEN_SHORTCUT));
        if ui.input_mut(|input| input.consume_shortcut(&SIDEBAR_SHORTCUT)) {
            self.is_sidebar_shown = !self.is_sidebar_shown;
        }

        let window_intent = self.top_bar_panel(ui);
        let sidebar_intent = match self.is_sidebar_shown {
            true => self.series_side_panel(ui),
            false => None,
        };
        let shown = self.develop_side_panel(ui);
        self.follow_histogram(&shown);
        let filmstrip_intent = if self.catalog.current().photos().is_empty() {
            None
        } else {
            self.filmstrip_panel(ui)
        };
        let mut zoom_shown = None;
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
                    zoom_shown = Some(zoom_readout(&output.view, output.scale));
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

        self.zoom_readout = zoom_shown;
        self.export_dialog_window(ui);
        if is_dialog_open {
            return;
        }
        if let Some(intent) = window_intent {
            self.apply_to_window(intent);
        }
        if let Some(intent) = sidebar_intent {
            self.apply_to_series(intent);
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
        eframe::set_value(storage, SIDEBAR_SHOWN_KEY, &self.is_sidebar_shown);
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
