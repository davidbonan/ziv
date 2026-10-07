use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{Key, KeyboardShortcut, Modifiers};

use crate::design::ui::floating_pill::PILL_STACKING_STEP;
use crate::design::ui::notice::{Notice, notice_toast};
use crate::design::ui::theme::{apply_theme, color, space};
use crate::develop::application::session_edits::{SaveFailure, SessionEdits};
use crate::develop::domain::aspect_ratio::RatioLock;
use crate::develop::domain::copied_edit::CopiedEdit;
use crate::develop::domain::development::Development;
use crate::develop::domain::edit::{Edit, FULL_INTENSITY};
use crate::develop::domain::framing::Framing;
use crate::develop::domain::mask::{Mask, MaskShape};
use crate::develop::domain::zone::ZoneMask;
use crate::develop::infrastructure::sidecar_files::{SidecarFiles, has_sidecar, sidecar_path};
use crate::develop::ui::before_badge::before_badge;
use crate::develop::ui::crop_canvas::{CropCanvasShown, crop_canvas};
use crate::develop::ui::crop_section::{CropTools, RatioAsked};
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
use crate::enhance::infrastructure::enhancement_files::enhancement_path;
use crate::enhance::infrastructure::photo_enhancement::enhance_photo;
use crate::enhance::ui::enhancement_progress::{enhancement_label, enhancement_progress};
use crate::export::application::export_run::ExportRun;
use crate::export::domain::export_settings::ExportSettings;
use crate::export::infrastructure::destination_picker::pick_destination;
use crate::export::infrastructure::photo_export::export_photo;
use crate::export::ui::export_dialog::{ExportDialogIntent, export_dialog};
use crate::export::ui::export_progress::{export_progress, export_summary};
use crate::histogram::infrastructure::developed_histogram::{CountedPhoto, DevelopedHistogram};
use crate::histogram::ui::histogram_plot::histogram_plot;
use crate::library::application::photo_trashing::trash_photos;
use crate::library::application::stored_catalog::StoredCatalog;
use crate::library::domain::catalog::Catalog;
use crate::library::domain::mark::{MOST_STARS, Rating};
use crate::library::domain::photo_selection::PhotoSelection;
use crate::library::domain::photo_trash::PhotoTrash;
use crate::library::domain::series::Series;
use crate::library::domain::series_filter::SeriesFilter;
use crate::library::infrastructure::catalog_file::CatalogFile;
use crate::library::infrastructure::photo_files::import_of;
use crate::library::infrastructure::photo_picker::{
    pick_photos_or_folder, pick_series_folder, show_in_finder,
};
use crate::library::infrastructure::series_covers::SeriesCovers;
use crate::library::infrastructure::system_trash::SystemTrash;
use crate::library::infrastructure::thumbnail_texture::thumbnail_texture;
use crate::library::infrastructure::today::today;
use crate::library::ui::empty_state::empty_state;
use crate::library::ui::filmstrip::{FILMSTRIP_HEIGHT, FilmstripIntent, filmstrip};
use crate::library::ui::mark_line::{MARK_LINE_HEIGHT, mark_line};
use crate::library::ui::no_photo_shown::no_photo_shown;
use crate::library::ui::photo_grid::{
    GridIntent, GridShown, NARROWEST_PHOTO_GRID, PHOTO_GRID_WIDTH, grid_header, photo_grid,
};
use crate::library::ui::photo_thumbnail::{PhotoThumbnail, ThumbnailState};
use crate::library::ui::series_sidebar::{
    SERIES_SIDEBAR_WIDTH, SeriesRow, SeriesShown, SidebarIntent, series_sidebar,
    unusable_catalog_warning,
};
use crate::library::ui::trash_confirmation::{TrashConfirmationIntent, trash_confirmation};
use crate::models::application::model_store::ModelStore;
use crate::models::infrastructure::model_downloads::ModelDownloads;
use crate::models::infrastructure::models_folder::models_folder;
use crate::models::infrastructure::onnx_runner::OnnxRunner;
use crate::photo::application::photo_loader::{Backlog, LoadedPhoto, PhotoLoader};
use crate::photo::application::photos_ahead::PhotosAhead;
use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::photo_details::PhotoDetails;
use crate::photo::domain::photo_kind::PhotoKind;
use crate::photo::domain::photo_name::photo_name;
use crate::photo::domain::shooting_data::ShootingData;
use crate::photo::domain::thumbnail::Thumbnail;
use crate::photo::infrastructure::file_decoder::FileDecoder;
use crate::photo::infrastructure::thumbnail_cache::ThumbnailCache;
use crate::photo::ui::photo_details_lines::{
    PHOTO_DETAILS_HEIGHT, PhotoDetailsShown, photo_details_lines,
};
use crate::photo::ui::shooting_data_line::shooting_data_line;
use crate::shell::domain::window_mode::WindowMode;
use crate::shell::ui::top_bar::{TOP_BAR_HEIGHT, TopBarIntent, TopBarShown, top_bar};
use crate::update::application::update_run::UpdateRun;
use crate::update::domain::update_state::UpdateState;
use crate::update::domain::version::Version;
use crate::update::domain::whats_new::WhatsNew;
use crate::update::infrastructure::app_update::AppUpdate;
use crate::update::infrastructure::relaunch::relaunch;
use crate::update::infrastructure::running_app_bundle::running_app_bundle;
use crate::update::ui::running_version_button::running_version_button;
use crate::update::ui::update_strip::{UpdateStripIntent, UpdateStripShown, update_strip};
use crate::update::ui::updates_dialog::{UpdatesDialogIntent, UpdatesShown, updates_dialog};
use crate::update::ui::whats_new::whats_new;
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
const NARROWEST_PREVIEW: f32 = 320.0;
const OPEN_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const EXPORT_SETTINGS_KEY: &str = "export settings";
const SIDEBAR_SHOWN_KEY: &str = "series sidebar shown";
const WINDOW_MODE_KEY: &str = "window mode";
const SEEN_VERSION_KEY: &str = "seen version";
const CULL_KEY: Key = Key::G;
const DEVELOP_KEY: Key = Key::D;
const REJECT_KEY: Key = Key::X;
/// The key that gives each rating, from no star to five.
const RATING_KEYS: [Key; MOST_STARS as usize + 1] = [
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
];
const SIDEBAR_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::L);
const EXPORT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::E);
const EXPORT_SESSION_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::E);
const BEFORE_KEY: Key = Key::B;
const CROP_KEY: Key = Key::R;
// In crop mode this key is the frame's, not the rejected mark's.
const SWAP_FRAME_SIDES_KEY: Key = Key::X;
const ROTATE_LEFT_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND, Key::OpenBracket);
const ROTATE_RIGHT_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND, Key::CloseBracket);
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
        path: PathBuf,
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

/// Crop mode on the viewed photo.
struct CropSession {
    /// The framing the photo had when crop mode was entered.
    framing_at_entry: Framing,
    tools: CropTools,
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
    source: Arc<SourceTexture>,
    kind: PhotoKind,
    shooting_data: ShootingData,
}

fn upload_decoded(engine: &Engine, path: &Path) -> Result<UploadedPhoto, DecodeError> {
    let decoded = FileDecoder.decode(path)?;
    Ok(UploadedPhoto {
        source: Arc::new(upload_with_enhancement(engine, path, &decoded.image)),
        kind: decoded.kind,
        shooting_data: decoded.shooting_data,
    })
}

struct PreviewedPhoto {
    photo: UploadedPhoto,
    details: PhotoDetails,
    /// The picture its RAW embeds, shown until the RAW is developed.
    is_stand_in: bool,
}

/// The photo as shot on the GPU, without waiting for a RAW to be developed
/// when the RAW embeds a picture.
fn upload_preview(engine: &Engine, path: &Path) -> Result<PreviewedPhoto, DecodeError> {
    let (photo, is_stand_in) = match FileDecoder.decode_embedded_picture(path)? {
        Some(picture) => {
            let photo = UploadedPhoto {
                source: Arc::new(engine.upload(&picture.image)),
                kind: picture.kind,
                shooting_data: picture.shooting_data,
            };
            (photo, true)
        }
        None => (upload_decoded(engine, path)?, false),
    };
    Ok(PreviewedPhoto {
        photo,
        details: FileDecoder.read_details(path),
        is_stand_in,
    })
}

fn develop_ahead(engine: &Engine, path: &Path) -> Result<PreviewedPhoto, DecodeError> {
    Ok(PreviewedPhoto {
        photo: upload_decoded(engine, path)?,
        details: FileDecoder.read_details(path),
        is_stand_in: false,
    })
}

pub struct ZivApp {
    egui_context: egui::Context,
    presenter: PhotoPresenter,
    viewed_loader: PhotoLoader<UploadedPhoto>,
    preview_loader: PhotoLoader<PreviewedPhoto>,
    /// What the file of the previewed photo says about it.
    photo_details: PhotoDetails,
    /// In Cull mode, the RAW being developed to replace the picture it embeds.
    awaited_development: Option<PathBuf>,
    /// In Cull mode, the photos next to the previewed one.
    ahead: PhotosAhead<PreviewedPhoto>,
    thumbnail_loader: PhotoLoader<Thumbnail>,
    thumbnail_cache: ThumbnailCache,
    covers: SeriesCovers,
    catalog: StoredCatalog<CatalogFile>,
    /// Why the series of this run will be forgotten, when what is stored cannot be used.
    catalog_warning: Option<String>,
    /// One per photo of the open series, in the same order.
    thumbnails: Vec<ThumbnailState>,
    is_sidebar_shown: bool,
    mode: WindowMode,
    /// What a mark or a removal acts on in the open series.
    selection: PhotoSelection,
    trash: Box<dyn PhotoTrash>,
    /// The photos the open confirmation is about to move to the Trash.
    photos_to_trash: Option<Vec<PathBuf>>,
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
    crop: Option<CropSession>,
    /// The framing the angle is being changed from, while a gesture changes it.
    straightened_from: Option<Framing>,
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
    /// `None` for an app that cannot update itself: it runs outside an app bundle.
    update: Option<UpdateRun>,
    /// An install was running at the last frame.
    was_update_installing: bool,
    is_relaunch_asked: bool,
    is_update_strip_dismissed: bool,
    is_updates_dialog_open: bool,
    is_whats_new_shown: bool,
    /// The last version whose release notes were shown or taken as the baseline.
    seen_version: Option<Version>,
    release_notes_layout: egui_commonmark::CommonMarkCache,
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
        let preview_loader = PhotoLoader::spawn(
            Backlog::LoadNewestOnly,
            {
                let engine = engine.clone();
                move |path| upload_preview(&engine, path)
            },
            {
                let egui_context = egui_context.clone();
                move || egui_context.request_repaint()
            },
        );
        let ahead = PhotosAhead::spawn(
            {
                let engine = engine.clone();
                move |path| develop_ahead(&engine, path)
            },
            || {},
        );
        let export_settings = creation
            .storage
            .and_then(|storage| eframe::get_value(storage, EXPORT_SETTINGS_KEY))
            .unwrap_or_default();
        let is_sidebar_shown = creation
            .storage
            .and_then(|storage| eframe::get_value(storage, SIDEBAR_SHOWN_KEY))
            .unwrap_or(true);
        let mode = creation
            .storage
            .and_then(|storage| eframe::get_value(storage, WINDOW_MODE_KEY))
            .unwrap_or_default();
        let stored_seen_version = creation
            .storage
            .and_then(|storage| eframe::get_value(storage, SEEN_VERSION_KEY));
        let update = running_app_bundle().map(|app_bundle| {
            let egui_context = egui_context.clone();
            UpdateRun::new(AppUpdate::of_ziv_in(app_bundle), move || {
                egui_context.request_repaint()
            })
        });
        let whats_new = match &update {
            Some(_) => WhatsNew::at_launch(Version::of_this_build(), stored_seen_version),
            None => WhatsNew::Nothing,
        };
        let catalog_path = catalog_file.path();
        let catalog = StoredCatalog::read_from(catalog_file);
        let mut app = Self {
            is_sidebar_shown,
            mode,
            selection: PhotoSelection::default(),
            trash: Box::new(SystemTrash),
            photos_to_trash: None,
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
            preview_loader,
            photo_details: PhotoDetails::default(),
            awaited_development: None,
            ahead,
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
            crop: None,
            straightened_from: None,
            mask_selection: MaskSelection::default(),
            copied_edit: None,
            update,
            was_update_installing: false,
            is_relaunch_asked: false,
            is_update_strip_dismissed: false,
            is_updates_dialog_open: false,
            is_whats_new_shown: whats_new == WhatsNew::Show,
            seen_version: match whats_new {
                WhatsNew::Nothing => stored_seen_version,
                WhatsNew::TakeAsSeen | WhatsNew::Show => Some(Version::of_this_build()),
            },
            release_notes_layout: egui_commonmark::CommonMarkCache::default(),
        };
        if let Some(update) = &app.update {
            update.check_quietly();
        }
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

    /// The app moving trashed files to `trash` instead of this Mac's Trash.
    pub fn trashing_with(mut self, trash: impl PhotoTrash + 'static) -> Self {
        self.trash = Box::new(trash);
        self
    }

    fn open(&mut self, paths: &[PathBuf]) {
        let import = import_of(paths, |path| FileDecoder.supports(path));
        let Some(series) = Series::imported(import, today()) else {
            return self.show_notice(NO_PHOTO_FOUND_NOTICE);
        };
        self.notice = None;
        let known_series = self.catalog.current().series().len();
        self.change_catalog(|catalog| catalog.import(series));
        if self.catalog.current().series().len() > known_series {
            self.mode = WindowMode::Cull;
        }
        self.show_open_series();
    }

    /// Each mode looks at its own picture of the selected photo.
    fn switch_to(&mut self, mode: WindowMode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.selection = PhotoSelection::default();
        self.view_selected_photo();
    }

    /// Rates or rejects the selection on the keys of the marks.
    fn mark_on_its_keys(&mut self, ui: &egui::Ui) {
        if ui.ctx().text_edit_focused() {
            return;
        }
        let is_pressed = |key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));
        let rating = (0..=MOST_STARS).find(|stars| is_pressed(RATING_KEYS[usize::from(*stars)]));
        if let Some(stars) = rating {
            self.rate_selection(Rating::of(stars));
        }
        if is_pressed(REJECT_KEY) {
            let photos = self.selected_photos();
            self.change_shown_photos(|catalog| catalog.toggle_rejected(&photos));
        }
    }

    fn rate_selection(&mut self, rating: Rating) {
        let photos = self.selected_photos();
        self.change_shown_photos(|catalog| catalog.rate(&photos, rating));
    }

    fn filter_series(&mut self, filter: SeriesFilter) {
        self.change_shown_photos(|catalog| catalog.set_filter(filter));
    }

    /// A change of marks or of filter: the photos shown may not be the same after it.
    fn change_shown_photos(&mut self, change: impl FnOnce(&mut Catalog)) {
        let shown_before = self.catalog.current().shown_photos();
        self.view_photo_selected_by(change);
        if self.catalog.current().shown_photos() != shown_before {
            self.selection = PhotoSelection::default();
        }
    }

    fn switch_mode_on_its_key(&mut self, ui: &egui::Ui) {
        if ui.ctx().text_edit_focused() {
            return;
        }
        let modes = [
            (CULL_KEY, WindowMode::Cull),
            (DEVELOP_KEY, WindowMode::Develop),
        ];
        for (key, mode) in modes {
            if ui.input_mut(|input| input.consume_key(Modifiers::NONE, key)) {
                self.switch_to(mode);
            }
        }
    }

    fn change_catalog(&mut self, change: impl FnOnce(&mut Catalog)) {
        if let Some(reason) = self.catalog.change(change) {
            self.show_notice(format!("The catalog could not be saved: {reason}"));
        }
    }

    fn show_open_series(&mut self) {
        self.selection = PhotoSelection::default();
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
        self.keep_viewed_photo_ahead();
        self.crop = None;
        let Some(path) = self.catalog.current().selected_photo() else {
            self.ahead.want(Vec::new());
            self.viewed = ViewedPhoto::None;
            return;
        };
        self.is_before_shown = false;
        self.photo_details = PhotoDetails::default();
        self.awaited_development = None;
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
        let path = path.to_owned();
        let developed_ahead = self.take_developed_ahead(&path);
        self.viewed = match (self.mode, developed_ahead) {
            (WindowMode::Cull, Some(developed)) => {
                self.photo_details = developed.details;
                self.ready_at_fit(path, developed.photo)
            }
            (WindowMode::Cull, None) => {
                self.preview_loader.request(path.clone());
                ViewedPhoto::Loading(path)
            }
            (WindowMode::Develop, _) => {
                self.viewed_loader.request(path.clone());
                ViewedPhoto::Loading(path)
            }
        };
    }

    /// The developed photo that stops being previewed is kept in case it is next to the one that follows.
    fn keep_viewed_photo_ahead(&mut self) {
        let ViewedPhoto::Ready {
            path,
            photo,
            kind,
            shooting_data,
            ..
        } = &self.viewed
        else {
            return;
        };
        let is_still_selected = self.catalog.current().selected_photo() == Some(path);
        if is_still_selected || self.awaited_development.is_some() {
            return;
        }
        let developed = PreviewedPhoto {
            photo: UploadedPhoto {
                source: photo.source().clone(),
                kind: *kind,
                shooting_data: *shooting_data,
            },
            details: self.photo_details,
            is_stand_in: false,
        };
        self.ahead.keep(path.clone(), developed);
    }

    /// In Cull mode the photos next to `selected` are developed ahead; in Develop mode none is.
    fn take_developed_ahead(&mut self, selected: &Path) -> Option<PreviewedPhoto> {
        let developed = self.ahead.take(selected);
        let beside = match self.mode {
            WindowMode::Cull => self.catalog.current().photos_beside_selected(),
            WindowMode::Develop => Vec::new(),
        };
        self.ahead
            .want(beside.into_iter().map(Path::to_owned).collect());
        developed
    }

    fn ready_at_fit(&self, path: PathBuf, uploaded: UploadedPhoto) -> ViewedPhoto {
        ViewedPhoto::Ready {
            path,
            photo: Box::new(self.presenter.present(uploaded.source)),
            kind: uploaded.kind,
            shooting_data: uploaded.shooting_data,
            histogram: DevelopedHistogram::default(),
            view: View::fit(),
        }
    }

    fn receive_viewed_photo(&mut self) {
        let (developed, previewed) = (
            self.viewed_loader.take_loaded(),
            self.preview_loader.take_loaded(),
        );
        match self.mode {
            WindowMode::Develop => developed
                .into_iter()
                .for_each(|loaded| self.show_loaded(loaded)),
            WindowMode::Cull => {
                previewed
                    .into_iter()
                    .for_each(|loaded| self.show_previewed(loaded));
                developed
                    .into_iter()
                    .for_each(|loaded| self.replace_stand_in(loaded));
            }
        }
    }

    /// The developed RAW takes the place of the picture it embeds, the same
    /// part of the photo staying on screen. A RAW that fails to develop keeps
    /// its picture.
    fn replace_stand_in(&mut self, LoadedPhoto { path, result }: LoadedPhoto<UploadedPhoto>) {
        if self.awaited_development.as_ref() != Some(&path) {
            return;
        }
        self.awaited_development = None;
        let (
            Ok(developed),
            ViewedPhoto::Ready {
                photo,
                kind,
                shooting_data,
                view,
                ..
            },
        ) = (result, &mut self.viewed)
        else {
            return;
        };
        let presented = self.presenter.present(developed.source);
        let enlargement = presented.size()[0] as f32 / photo.size()[0] as f32;
        *view = view.for_photo_scaled_by(enlargement);
        **photo = presented;
        *kind = developed.kind;
        *shooting_data = developed.shooting_data;
    }

    fn is_awaited(&self, path: &Path) -> bool {
        matches!(&self.viewed, ViewedPhoto::Loading(awaited) if awaited == path)
    }

    fn show_previewed(&mut self, LoadedPhoto { path, result }: LoadedPhoto<PreviewedPhoto>) {
        if !self.is_awaited(&path) {
            return;
        }
        let mut is_stand_in = false;
        let result = result.map(|previewed| {
            self.photo_details = previewed.details;
            is_stand_in = previewed.is_stand_in;
            previewed.photo
        });
        if is_stand_in {
            self.viewed_loader.request(path.clone());
            self.awaited_development = Some(path.clone());
        }
        self.show_loaded(LoadedPhoto { path, result });
    }

    fn show_loaded(&mut self, loaded: LoadedPhoto<UploadedPhoto>) {
        if !self.is_awaited(&loaded.path) {
            return;
        }
        self.viewed = match loaded.result {
            Ok(uploaded) => self.ready_at_fit(loaded.path, uploaded),
            Err(error) => {
                self.mark_thumbnail_failed(&loaded.path);
                ViewedPhoto::Failed {
                    path: loaded.path,
                    error,
                }
            }
        };
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
        match intent {
            FilmstripIntent::Select(position) => self.select_shown_photo(position),
            FilmstripIntent::SelectPrevious => self.select_photo(Catalog::select_previous),
            FilmstripIntent::SelectNext => self.select_photo(Catalog::select_next),
        }
    }

    /// Selects alone the photo at `position` among the shown ones.
    fn select_shown_photo(&mut self, position: usize) {
        if let Some(photo) = self.catalog.current().shown_photos().get(position).copied() {
            self.select_photo(|catalog| catalog.select(photo));
        }
    }

    /// The grid speaks of photos by their place among the shown ones.
    fn apply_to_grid(&mut self, intent: GridIntent) {
        let catalog = self.catalog.current();
        let shown = catalog.shown_photos();
        let selected = catalog.selected_index();
        match intent {
            GridIntent::Select(position) => self.select_shown_photo(position),
            GridIntent::Toggle(position) => {
                let (Some(photo), Some(selected)) = (shown.get(position), selected) else {
                    return;
                };
                let to_select = self.selection.toggle(*photo, selected);
                self.view_photo_selected_by(|catalog| catalog.select(to_select));
            }
            GridIntent::ExtendTo(position) => {
                let from =
                    selected.and_then(|selected| shown.iter().position(|photo| *photo == selected));
                let (Some(from), Some(photo)) = (from, shown.get(position).copied()) else {
                    return;
                };
                let between = &shown[from.min(position)..=from.max(position)];
                self.selection = PhotoSelection::of(between.iter().copied());
                self.view_photo_selected_by(|catalog| catalog.select(photo));
            }
            GridIntent::SelectAll => self.selection = PhotoSelection::of(shown),
            GridIntent::Open(position) => {
                self.select_shown_photo(position);
                self.switch_to(WindowMode::Develop);
            }
            GridIntent::RemoveSelection => self.remove_from_series(self.selected_photos()),
            GridIntent::RemoveFromSeries(position) => {
                if let Some(photo) = shown.get(position) {
                    self.remove_from_series(self.selection_or(*photo));
                }
            }
            GridIntent::TrashSelection => self.ask_to_trash(self.selected_photos()),
            GridIntent::TrashRejected => self.ask_to_trash(catalog.rejected_photos()),
            GridIntent::MoveToTrash(position) => {
                if let Some(photo) = shown.get(position) {
                    self.ask_to_trash(self.selection_or(*photo));
                }
            }
        }
    }

    fn ask_to_trash(&mut self, photos: Vec<usize>) {
        let of_series = self.catalog.current().photos();
        let files: Vec<PathBuf> = photos
            .into_iter()
            .filter_map(|photo| of_series.get(photo).cloned())
            .collect();
        if !files.is_empty() {
            self.photos_to_trash = Some(files);
        }
    }

    fn trash_confirmation_window(&mut self, ui: &egui::Ui) {
        let Some(photos) = &self.photos_to_trash else {
            return;
        };
        let modal = egui::Modal::new(egui::Id::new("trash confirmation"))
            .frame(egui::Frame::window(ui.style()).inner_margin(space::L))
            .show(ui.ctx(), |ui| trash_confirmation(ui, photos.len()));
        match modal.inner {
            Some(TrashConfirmationIntent::Confirm) => self.trash_asked_photos(),
            Some(TrashConfirmationIntent::Cancel) => self.photos_to_trash = None,
            None if modal.should_close() => self.photos_to_trash = None,
            None => {}
        }
    }

    /// Moves the photos of the confirmation to the Trash, each with its
    /// sidecar and its enhancement file, and takes them out of the series.
    fn trash_asked_photos(&mut self) {
        let Some(photos) = self.photos_to_trash.take() else {
            return;
        };
        let failure = self.edits.save_now();
        self.report(failure);
        let outcome = trash_photos(self.trash.as_ref(), &photos, |photo| {
            let beside = [sidecar_path(photo), enhancement_path(photo)];
            beside.into_iter().filter(|file| file.exists()).collect()
        });
        let catalog = self.catalog.current();
        let trashed = outcome.trashed.iter();
        let left: Vec<usize> = trashed
            .filter_map(|photo| catalog.index_of(photo))
            .collect();
        self.change_catalog(|catalog| catalog.drop_trashed_photos(&left));
        self.show_open_series();
        match outcome.failed_count {
            0 => {}
            1 => self.show_notice("1 photo could not be moved to the Trash"),
            count => self.show_notice(format!("{count} photos could not be moved to the Trash")),
        }
    }

    /// The selection when `photo` is in it, else `photo` alone: what a menu of that photo acts on.
    fn selection_or(&self, photo: usize) -> Vec<usize> {
        let selection = self.selected_photos();
        match selection.contains(&photo) {
            true => selection,
            false => vec![photo],
        }
    }

    fn remove_from_series(&mut self, photos: Vec<usize>) {
        self.change_catalog(|catalog| catalog.remove_photos(&photos));
        self.show_open_series();
    }

    /// Selects one photo alone.
    fn select_photo(&mut self, select: impl FnOnce(&mut Catalog)) {
        self.selection = PhotoSelection::default();
        self.view_photo_selected_by(select);
    }

    fn view_photo_selected_by(&mut self, select: impl FnOnce(&mut Catalog)) {
        let previously_selected = self.catalog.current().selected_index();
        self.change_catalog(select);
        if self.catalog.current().selected_index() != previously_selected {
            self.view_selected_photo();
        }
    }

    fn apply_to_window(&mut self, intent: TopBarIntent) {
        match intent {
            TopBarIntent::ToggleSidebar => self.is_sidebar_shown = !self.is_sidebar_shown,
            TopBarIntent::SwitchTo(mode) => self.switch_to(mode),
            TopBarIntent::Filter(filter) => self.filter_series(filter),
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
        if self.mode != WindowMode::Develop || self.is_cropping() {
            return;
        }
        if !is_typing && ui.input_mut(|input| input.consume_key(Modifiers::NONE, BEFORE_KEY)) {
            self.is_before_shown = !self.is_before_shown;
        }
    }

    fn is_cropping(&self) -> bool {
        self.crop.is_some()
    }

    fn can_crop(&self) -> bool {
        self.mode == WindowMode::Develop
            && matches!(self.viewed, ViewedPhoto::Ready { .. })
            && self.edits_left_alone_reason().is_none()
    }

    fn enter_crop_mode(&mut self) {
        let ViewedPhoto::Ready { photo, .. } = &self.viewed else {
            return;
        };
        if !self.can_crop() {
            return;
        }
        let framing = self.selected_edit().framing;
        self.crop = Some(CropSession {
            framing_at_entry: framing,
            tools: CropTools {
                ratio: RatioLock::on_frame(&framing, photo.size()),
                is_level_armed: false,
            },
        });
        self.is_before_shown = false;
        self.mask_selection = self.mask_selection.selecting(None);
    }

    /// Leaves crop mode keeping the framing; the framed photo is shown at fit.
    fn leave_crop_mode(&mut self) {
        self.crop = None;
        if let ViewedPhoto::Ready { view, .. } = &mut self.viewed {
            *view = View::fit();
        }
    }

    /// Leaves crop mode with the framing the photo had when it was entered.
    fn cancel_crop(&mut self) {
        let Some(CropSession {
            framing_at_entry, ..
        }) = self.crop
        else {
            return;
        };
        self.set_framing(framing_at_entry);
        self.leave_crop_mode();
    }

    /// `Esc` gives up the level tool when it is armed, else the framing.
    fn disarm_level_or_cancel_crop(&mut self) {
        match &mut self.crop {
            Some(crop) if crop.tools.is_level_armed => crop.tools.is_level_armed = false,
            _ => self.cancel_crop(),
        }
    }

    fn set_framing(&mut self, framing: Framing) {
        self.set_selected_edit(Edit {
            framing,
            ..self.selected_edit()
        });
    }

    /// Gives the crop frame the proportions asked for, or lets them go.
    fn hold_ratio(&mut self, asked: RatioAsked) {
        let ViewedPhoto::Ready { photo, .. } = &self.viewed else {
            return;
        };
        let picture = photo.size();
        let framing = self.selected_edit().framing;
        let (ratio, framed) = match asked {
            RatioAsked::Free => (RatioLock::Free, framing),
            RatioAsked::OfFrame => (RatioLock::on_frame(&framing, picture), framing),
            RatioAsked::Named(named) => (
                RatioLock::Named(named),
                framing.with_ratio(picture, named.long_over_short(picture)),
            ),
        };
        if let Some(crop) = &mut self.crop {
            crop.tools.ratio = ratio;
        }
        self.set_framing(framed);
    }

    /// Turns the crop frame from landscape to portrait or back.
    fn swap_frame_sides(&mut self) {
        if let ViewedPhoto::Ready { photo, .. } = &self.viewed {
            let swapped = self
                .selected_edit()
                .framing
                .with_sides_swapped(photo.size());
            self.set_framing(swapped);
        }
    }

    /// `edit` with its frame held in the picture when only its angle was
    /// changed: the frame is the one the gesture started with, shrunk for that angle.
    fn with_frame_held_in_picture(&mut self, edit: Edit) -> Edit {
        let current = self.selected_edit().framing;
        let ViewedPhoto::Ready { photo, .. } = &self.viewed else {
            return edit;
        };
        if edit.framing.angle == current.angle || edit.framing.frame != current.frame {
            return edit;
        }
        let from = *self.straightened_from.get_or_insert(current);
        Edit {
            framing: from.straightened(photo.size(), edit.framing.angle),
            ..edit
        }
    }

    /// `Cmd+[` and `Cmd+]` turn the photo by a quarter turn, in crop mode or not.
    fn turn_photo_on_shortcut(&mut self, ui: &egui::Ui) {
        if !self.can_crop() {
            return;
        }
        let framing = self.selected_edit().framing;
        let turn = if ui.input_mut(|input| input.consume_shortcut(&ROTATE_LEFT_SHORTCUT)) {
            framing.turn.turned_left()
        } else if ui.input_mut(|input| input.consume_shortcut(&ROTATE_RIGHT_SHORTCUT)) {
            framing.turn.turned_right()
        } else {
            return;
        };
        self.set_framing(Framing { turn, ..framing });
    }

    fn set_cropping(&mut self, is_cropping: bool) {
        match (self.is_cropping(), is_cropping) {
            (false, true) => self.enter_crop_mode(),
            (true, false) => self.leave_crop_mode(),
            _ => {}
        }
    }

    /// `R` enters crop mode and leaves it, as `Enter` does; `Esc` gives the framing up.
    fn crop_on_its_keys(&mut self, ui: &egui::Ui) {
        if !ui.input(|input| input.pointer.any_down()) {
            self.straightened_from = None;
        }
        if ui.ctx().text_edit_focused() {
            return;
        }
        let is_pressed = |key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));
        if is_pressed(CROP_KEY) {
            return self.set_cropping(!self.is_cropping());
        }
        if !self.is_cropping() {
            return;
        }
        if is_pressed(Key::Enter) {
            self.leave_crop_mode();
        } else if is_pressed(Key::Escape) {
            self.disarm_level_or_cancel_crop();
        } else if is_pressed(SWAP_FRAME_SIDES_KEY) {
            self.swap_frame_sides();
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
            let catalog = self.catalog.current();
            let shown = catalog.shown_photos().into_iter();
            self.ask_to_export(shown.map(|photo| catalog.photos()[photo].clone()).collect());
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
        self.mode == WindowMode::Develop
            && !self.is_before_shown
            && !self.is_cropping()
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
        person_outlines(ui, photo.area, |share| photo.share_on_screen(share), pick);
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

    fn can_update_be_installed(&self) -> bool {
        self.export_run.is_none() && self.enhancement.is_none()
    }

    /// Relaunches once the new version is in place; says why an install failed.
    fn follow_update(&mut self, frame: &mut eframe::Frame) {
        let Some(update) = &self.update else {
            return;
        };
        let state = update.state();
        let was_installing = self.was_update_installing;
        self.was_update_installing = state.is_installing();
        match state {
            UpdateState::Installed(app_bundle) if !self.is_relaunch_asked => {
                self.is_relaunch_asked = true;
                self.relaunch_into(&app_bundle, frame);
            }
            UpdateState::Failed(reason) if was_installing => self.show_notice(reason),
            _ => {}
        }
    }

    fn relaunch_into(&mut self, app_bundle: &Path, frame: &mut eframe::Frame) {
        let failure = self.edits.save_now();
        self.report(failure);
        if let Some(storage) = frame.storage_mut() {
            eframe::App::save(self, storage);
            storage.flush();
        }
        match relaunch(app_bundle) {
            Ok(()) => self
                .egui_context
                .send_viewport_cmd(egui::ViewportCommand::Close),
            Err(error) => self.show_notice(format!(
                "The new version is installed but could not be launched: {error}"
            )),
        }
    }

    /// Above the places of the other messages.
    fn update_strip_over(&mut self, ui: &egui::Ui) {
        let Some(update) = &self.update else {
            return;
        };
        let state = update.state();
        if self.is_update_strip_dismissed && !state.is_installing() {
            return;
        }
        let shown = UpdateStripShown {
            state: &state,
            can_install: self.can_update_be_installed(),
        };
        let over = ui
            .max_rect()
            .translate(egui::vec2(0.0, -2.0 * PILL_STACKING_STEP));
        match update_strip(ui, over, &shown) {
            Some(UpdateStripIntent::Install) => update.install(),
            Some(UpdateStripIntent::Later) => self.is_update_strip_dismissed = true,
            None => {}
        }
    }

    fn updates_dialog_window(&mut self, ui: &egui::Ui) {
        if !self.is_updates_dialog_open {
            return;
        }
        let state = self.update.as_ref().map(UpdateRun::state);
        let shown = UpdatesShown {
            running: Version::of_this_build(),
            state: state.as_ref(),
            can_install: self.can_update_be_installed(),
        };
        let modal = egui::Modal::new(egui::Id::new("updates dialog"))
            .frame(egui::Frame::window(ui.style()).inner_margin(space::L))
            .show(ui.ctx(), |ui| {
                updates_dialog(ui, &shown, &mut self.release_notes_layout)
            });
        match (modal.inner, &self.update) {
            (Some(UpdatesDialogIntent::Check), Some(update)) => update.check(),
            (Some(UpdatesDialogIntent::Install), Some(update)) => update.install(),
            (Some(UpdatesDialogIntent::Close), _) => self.is_updates_dialog_open = false,
            (None, _) if modal.should_close() => self.is_updates_dialog_open = false,
            _ => {}
        }
    }

    fn whats_new_window(&mut self, ui: &egui::Ui) {
        if !self.is_whats_new_shown {
            return;
        }
        let modal = egui::Modal::new(egui::Id::new("what's new"))
            .frame(egui::Frame::window(ui.style()).inner_margin(space::L))
            .show(ui.ctx(), |ui| whats_new(ui, &mut self.release_notes_layout));
        self.is_whats_new_shown = !modal.inner && !modal.should_close();
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
                    histogram: histogram.cloned(),
                    mask_selection: self.mask_selection,
                    is_before_shown: self.is_before_shown,
                    crop: self.crop.as_ref().map(|crop| crop.tools),
                    ratio_asked: None,
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
        let mut left = left;
        if ready_kind.is_some() {
            left.edit = self.with_frame_held_in_picture(left.edit);
            self.set_selected_edit(left.edit.clone());
            self.mask_selection = left.mask_selection;
            self.is_before_shown = left.is_before_shown;
            self.set_cropping(left.crop.is_some());
            if let (Some(crop), Some(tools)) = (&mut self.crop, left.crop) {
                crop.tools.is_level_armed = tools.is_level_armed;
            }
            if let Some(asked) = left.ratio_asked {
                self.hold_ratio(asked);
                left.edit.framing = self.selected_edit().framing;
            }
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
                framing: left.edit.framing,
                overlaid_mask: None,
            };
        }
        ShownPhoto {
            framing: left.edit.framing,
            overlaid_mask: left.mask_selection.overlaid_mask(&left.edit).cloned(),
            development: Development {
                kind,
                edit: left.edit,
            },
        }
    }

    /// The viewed photo without its edit, as Cull mode shows it.
    fn photo_as_shot(&self) -> ShownPhoto {
        let ViewedPhoto::Ready { kind, .. } = &self.viewed else {
            return ShownPhoto::default();
        };
        ShownPhoto {
            development: Development {
                kind: *kind,
                edit: Edit::default(),
            },
            ..ShownPhoto::default()
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
        let counted = CountedPhoto {
            development: shown.development.clone(),
            framing: shown.framing,
        };
        if histogram.follow(&self.engine, photo.source(), &counted) {
            self.egui_context.request_repaint();
        }
    }

    fn series_side_panel(&mut self, ui: &mut egui::Ui) -> Option<SidebarIntent> {
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
                self.is_updates_dialog_open |= egui::Panel::bottom("running version")
                    .frame(egui::Frame::new())
                    .show(ui, |ui| {
                        running_version_button(ui, Version::of_this_build()).clicked()
                    })
                    .inner;
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
            mode: self.mode,
            filter: catalog.open_series().map(|series| series.filter),
            zoom_readout: self.zoom_readout.as_deref(),
            is_before_shown: self.is_before_shown,
            can_before_be_shown: self.mode == WindowMode::Develop
                && !self.is_cropping()
                && matches!(self.viewed, ViewedPhoto::Ready { .. }),
            can_export: catalog.selected_photo().is_some(),
        };
        egui::Panel::top("top bar")
            .exact_size(TOP_BAR_HEIGHT)
            .frame(panel_frame(space::S))
            .show(ui, |ui| top_bar(ui, &shown))
            .inner
    }

    /// The shown photos of the open series, as the filmstrip and the grid show them.
    fn photo_thumbnails(&self) -> Vec<PhotoThumbnail<'_>> {
        let catalog = self.catalog.current();
        let shown = catalog.shown_photos().into_iter();
        shown
            .map(|photo| {
                let path = &catalog.photos()[photo];
                PhotoThumbnail {
                    path,
                    thumbnail: &self.thumbnails[photo],
                    is_edited: self.edited_marks[photo],
                    mark: catalog.mark_of(path),
                }
            })
            .collect()
    }

    /// Where the selected photo is among the shown ones.
    fn selected_position(&self) -> Option<usize> {
        let catalog = self.catalog.current();
        let selected = catalog.selected_index()?;
        let mut shown = catalog.shown_photos().into_iter();
        shown.position(|photo| photo == selected)
    }

    /// Returns what the user asked of the grid, and whether to show every photo.
    fn grid_panel(&self, ui: &mut egui::Ui) -> (Option<GridIntent>, bool) {
        let photos = self.photo_thumbnails();
        let catalog = self.catalog.current();
        let is_in_selection: Vec<bool> = match catalog.selected_index() {
            Some(selected) => {
                let shown = catalog.shown_photos().into_iter();
                shown
                    .map(|photo| self.selection.contains(photo, selected))
                    .collect()
            }
            None => Vec::new(),
        };
        let margins = 2.0 * space::M;
        let widest = ui.available_width() - NARROWEST_PREVIEW;
        egui::Panel::left("photo grid")
            .resizable(true)
            .default_size(PHOTO_GRID_WIDTH + margins)
            .size_range(NARROWEST_PHOTO_GRID + margins..=widest)
            .frame(panel_frame(space::M))
            .show(ui, |ui| {
                let has_rejected_photos = !catalog.rejected_photos().is_empty();
                let trash_rejected =
                    grid_header(ui, has_rejected_photos).then_some(GridIntent::TrashRejected);
                if photos.is_empty() {
                    return (trash_rejected, no_photo_shown(ui, catalog.filter()));
                }
                let shown = GridShown {
                    photos: &photos,
                    selected: self.selected_position(),
                    is_in_selection: &is_in_selection,
                };
                (photo_grid(ui, &shown).or(trash_rejected), false)
            })
            .inner
    }

    /// The photos of the selection, by their place in the open series.
    fn selected_photos(&self) -> Vec<usize> {
        let selected = self.catalog.current().selected_index();
        selected.map_or_else(Vec::new, |selected| self.selection.photos(selected))
    }

    /// Returns the rating a clicked star asks for.
    fn photo_details_panel(&self, ui: &mut egui::Ui) -> Option<Rating> {
        let photo = self.catalog.current().selected_photo()?;
        let shooting_data = match &self.viewed {
            ViewedPhoto::Ready { shooting_data, .. } => *shooting_data,
            _ => ShootingData::default(),
        };
        let shown = PhotoDetailsShown {
            name: &photo_name(photo),
            shooting_data: &shooting_data,
            details: &self.photo_details,
            selection_size: self.selected_photos().len(),
        };
        let mark = self.catalog.current().mark_of(photo);
        egui::Panel::bottom("photo details")
            .exact_size(PHOTO_DETAILS_HEIGHT + MARK_LINE_HEIGHT + space::XS)
            .frame(panel_frame(space::M))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = space::XS;
                let rating_asked = mark_line(ui, mark);
                photo_details_lines(ui, &shown);
                rating_asked
            })
            .inner
    }

    fn filmstrip_panel(&self, ui: &mut egui::Ui) -> Option<FilmstripIntent> {
        let photos = self.photo_thumbnails();
        egui::Panel::bottom("filmstrip")
            .exact_size(FILMSTRIP_HEIGHT)
            .frame(panel_frame(space::M))
            .show(ui, |ui| filmstrip(ui, &photos, self.selected_position()))
            .inner
    }
}

impl eframe::App for ZivApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.follow_update(frame);
        self.receive_viewed_photo();
        self.receive_thumbnails();
        self.covers.receive_loaded();
        self.receive_detection();
        self.receive_enhancement();
        self.open_dropped_files(ui);
        let is_dialog_open = self.photos_to_export.is_some()
            || self.photos_to_trash.is_some()
            || self.is_updates_dialog_open
            || self.is_whats_new_shown;
        if !is_dialog_open {
            self.undo_or_redo(ui);
            self.crop_on_its_keys(ui);
            self.turn_photo_on_shortcut(ui);
            self.toggle_before_on_its_key(ui);
            self.copy_or_paste_edit_on_shortcut(ui);
            self.ask_to_export_on_shortcut(ui);
            self.switch_mode_on_its_key(ui);
            self.mark_on_its_keys(ui);
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
        let has_photos = !self.catalog.current().photos().is_empty();
        let has_shown_photos = !self.catalog.current().shown_photos().is_empty();
        let filter = self.catalog.current().filter();
        let mut is_show_all_asked = false;
        let is_developing = self.mode == WindowMode::Develop;
        let mut filmstrip_intent = None;
        let mut grid_intent = None;
        let mut rating_asked = None;
        let shown = match self.mode {
            WindowMode::Develop => {
                let shown = self.develop_side_panel(ui);
                self.follow_histogram(&shown);
                if has_shown_photos {
                    filmstrip_intent = self.filmstrip_panel(ui);
                }
                shown
            }
            WindowMode::Cull => {
                if has_photos {
                    (grid_intent, is_show_all_asked) = self.grid_panel(ui);
                    rating_asked = self.photo_details_panel(ui);
                }
                self.photo_as_shot()
            }
        };
        let mut zoom_shown = None;
        let mut masks_on_screen = None;
        let mut cropped = None;
        let is_cropping = self.is_cropping();
        let crop_tools = self.crop.as_ref().map(|crop| crop.tools);
        let canvas = egui::Frame::new().fill(color::CANVAS);
        egui::CentralPanel::default().frame(canvas).show(ui, |ui| {
            self.notice_over(ui);
            self.export_progress_over(ui);
            self.detection_status_over(ui);
            self.enhancement_progress_over(ui);
            self.update_strip_over(ui);
            match &mut self.viewed {
                ViewedPhoto::None if !has_photos => open_requested |= empty_state(ui),
                ViewedPhoto::None if is_developing => {
                    is_show_all_asked |= no_photo_shown(ui, filter)
                }
                ViewedPhoto::None => {}
                ViewedPhoto::Loading(path) => photo_loading(ui, path),
                ViewedPhoto::Ready { photo, .. } if is_cropping => {
                    let picture = photo.size();
                    let whole_picture =
                        |region, size| self.presenter.render(photo, shown.request_of(region, size));
                    let canvas = CropCanvasShown {
                        picture,
                        framing: shown.framing,
                        tools: crop_tools.unwrap_or_default(),
                    };
                    cropped = Some(crop_canvas(ui, &canvas, whole_picture));
                }
                ViewedPhoto::Ready { photo, view, .. } => {
                    let picture = photo.size();
                    let framed = shown.framing.framed_size(picture);
                    let output = photo_viewport(ui, framed, *view, |placement| {
                        self.presenter
                            .render(photo, shown.request_at(picture, placement))
                    });
                    *view = output.view;
                    zoom_shown = Some(zoom_readout(&output.view, output.scale));
                    masks_on_screen = is_developing.then_some(PhotoOnScreen::framed(
                        ui.max_rect(),
                        output.whole_photo_rect,
                        (&shown.framing, picture),
                    ));
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
        if let Some(cropped) = cropped {
            self.set_selected_edit(Edit {
                framing: cropped.framing,
                ..self.selected_edit()
            });
            if let Some(crop) = self.crop.as_mut().filter(|_| cropped.is_level_drawn) {
                crop.tools.is_level_armed = false;
            }
            if cropped.is_done {
                self.leave_crop_mode();
            }
        }

        self.zoom_readout = zoom_shown;
        self.export_dialog_window(ui);
        self.trash_confirmation_window(ui);
        self.updates_dialog_window(ui);
        self.whats_new_window(ui);
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
        if let Some(intent) = grid_intent {
            self.apply_to_grid(intent);
        }
        if let Some(rating) = rating_asked {
            self.rate_selection(rating);
        }
        if is_show_all_asked {
            self.filter_series(SeriesFilter::All);
        }
        if open_requested {
            self.open_picked_photos();
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, EXPORT_SETTINGS_KEY, &self.export_settings);
        eframe::set_value(storage, SIDEBAR_SHOWN_KEY, &self.is_sidebar_shown);
        eframe::set_value(storage, WINDOW_MODE_KEY, &self.mode);
        if let Some(seen) = self.seen_version {
            eframe::set_value(storage, SEEN_VERSION_KEY, &seen);
        }
    }

    fn on_exit(&mut self) {
        let _ = self.edits.save_now();
    }
}

pub fn run() -> eframe::Result<()> {
    eframe::run_native(
        APP_NAME,
        eframe::NativeOptions::default(),
        Box::new(|creation| {
            #[cfg(target_os = "macos")]
            crate::shell::infrastructure::edit_menu::install_edit_menu();
            Ok(Box::new(ZivApp::new(creation)))
        }),
    )
}
