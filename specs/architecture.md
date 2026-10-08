# ziv — Architecture

## 1. Boundary
- **Domain** (image math, edit model, masks, RAW decoding, export): no `egui`,
  no `eframe`. `pub` from the lib.
- **Engine**: turns *source image + edit parameters* into pixels on `wgpu`.
  Takes a `wgpu::Device`, never a window — the same code path serves the
  viewport, the export and the tests.
- **UI**: `pub fn(&mut egui::Ui, …)` render functions. They read state and emit
  intents; they hold no business rule.
- **`app.rs`**: wires the modules to `eframe`. Kept thin.

## 2. Folder rule
Module → part of the feature → technical layer.

```
src/<module>/<part>/{domain,application,infrastructure,ui}/
```

- A module is a domain concept (`library`, …). Never `utils/`, `helpers/`, `common/`.
- Layers are leaves. `infrastructure/` and `ui/` may import `domain/`, never the
  reverse; `application/` depends only on ports declared in `domain/`.
- A module with a single part keeps its layers directly under the module
  (`src/library/ui/`) until a second part exists.

## 3. Current tree
```
src/main.rs                      eframe wrapper
src/lib.rs                       module declarations
src/app.rs                       ZivApp, run()
src/design/ui/                   theme (colors, hues, spacing, type), AdjustmentSlider (accent, hue or color track), icon button and toggle,
                                 primary button, folding section, marked choice (a selector button, marked when it holds an edit), floating pill, Notice
src/shell/domain/                WindowMode: Cull or Develop
src/shell/infrastructure/        native Edit menu of the menu bar (`objc2-app-kit`)
src/shell/ui/                    top bar: sidebar button, series and photo names, mode switch, zoom readout, Before, Export…
src/color/domain/                primaries, matrices, sRGB transfer, working space, Oklab, Illuminant
src/photo/application/           PhotoLoader (worker thread), PhotosAhead (photos loaded before they are asked for)
src/photo/domain/                PictureRegion (the parallelogram of a picture a render shows), WorkingImage, DecodedPhoto, PhotoKind, ShootingData, PhotoDetails (pixel size, ShotAt, file size), Thumbnail, Orientation, CameraCalibration, DecodeError
src/photo/infrastructure/        file decoders: standard images (`image`), RAW (`rawler`), the picture a RAW embeds as its preview; shooting data from EXIF (`rawler`);
                                 ThumbnailCache (JPEG files in the cache folder)
src/photo/ui/                    shooting data line, photo details lines
src/histogram/domain/            Histogram: luminance and channel counts per display level, heights, sample size
src/histogram/infrastructure/    DevelopedHistogram: the photo rendered small by the engine and counted, once per development
src/histogram/ui/                histogram plot
src/develop/application/         SessionEdits: edits of the session, stored as they settle
src/develop/infrastructure/      SidecarFiles: one JSON file next to each edited photo
src/develop/domain/              Edit (Adjustments + masks + tone curves + color mixer + color grading), EditHistory, CopiedEdit, WhiteBalance, Tone, Presence, BaseRendering, Development;
                                 ToneCurve (points, monotone interpolation, lookup), one per CurveChannel;
                                 ColorMixer (Hue, Saturation, Luminance per ColorRange); ColorGrading (a ZoneGrade per TonalZone, Blending, Balance);
                                 Framing (CropFrame and its handles, angle, Turn: mirror and quarter turns), NamedRatio and RatioLock, CropView (the whole picture laid out in crop mode);
                                 Mask and its shapes (LinearGradient, RadialGradient, Rectangle, Polygon, BrushMask, ZoneMask), BrushCoverage, CoverageImage, overlay;
                                 Preset (the zones it masks, the adjustments it sets on each), PresetGroup, AppliedPreset (carried by the masks a preset made)
src/develop/ui/                  develop panel (tool bar, folding sections, foot), Presets section, mask tool bar and masks list (applied presets above their masks),
                                 mask canvas (drawing and handles over the photo),
                                 crop canvas (the whole picture, its crop frame and handles, level line), Crop section (ratios, angle, turns, Reset crop), tone curve section (channel selector, graph, Reset curve),
                                 color mixer section (adjustment selector, eight sliders with color tracks),
                                 color wheel, color grading section (3-way and zone views, Blending, Balance), Before badge
src/export/domain/               ExportSettings: format, size, destination, file naming
src/export/application/          ExportRun: several photos exported in the background
src/export/infrastructure/       one photo → file (`image` encoders), destination picker
src/export/ui/                   export dialog, progress, summary
src/engine/infrastructure/       Engine: upload + mipmaps, display stage (WGSL: enhancement mix, adjustments, masks, color mixer, color grading, tone curve, overlay), mask coverage layers, curve lookup, readback, render to pixels
src/enhance/domain/              Enhancement (differences from the original), ModelEncoding, noise level, overlapping tiles, unsharp mask, the enhancement model; port: EnhancementStorage
src/enhance/application/         Enhancer: photo → enhancement, tile by tile; EnhancementRun (worker thread, Cancel)
src/enhance/infrastructure/      EnhancementFiles: one file next to each enhanced photo; photo → enhancement file; upload with enhancement
src/enhance/ui/                  Detail section (Enhance, Intensity), enhancement progress
src/viewport/domain/             View: fit, zoom, pan → Placement; zoom readout
src/viewport/infrastructure/     PhotoPresenter: engine output → egui texture
src/viewport/ui/                 photo_viewport, loading / failed status
src/library/domain/              Catalog, Series (Import, ImportDay), Session, natural order, GridStep, PhotoSelection, Mark and Rating, SeriesFilter,
                                 the catalog document; ports: CatalogStorage, PhotoTrash
src/library/application/         StoredCatalog: the catalog, stored as it changes; photos moved to the Trash with the files beside them
src/library/infrastructure/      photo files among opened paths, native pickers, Finder, CatalogFile (one JSON file in the data folder), today,
                                 SeriesCovers (cover thumbnails → egui textures, worker thread), SystemTrash (`trash`)
src/library/ui/                  empty state, photo thumbnail (shared cell painting), filmstrip, photo grid and its header, mark line,
                                 series filter switch, no photo shown, trash confirmation, series sidebar
src/models/domain/               Model (its files: address, checksum, size); ports: ModelSource, ModelRunner
src/models/application/          ModelStore: download at first use, checked, kept
src/models/infrastructure/       HTTPS downloads, models folder, ONNX Runtime runner (`ort`): CPU, or GPU for a model of fixed shape
src/zones/domain/                the zone models, model input, mattes and their refinement on the photo's outlines,
                                 persons and their parts, the teeth of a mouth, PeoplePick; port: PhotoView
src/zones/application/           ZoneDetector: photo → zone masks, persons, person parts, the zones of a preset; DetectionRun (worker thread)
src/zones/ui/                    detection status and its messages, people picker and person outlines
src/zones/infrastructure/        photo view rendered by the engine
src/update/domain/               Version, PublishedRelease (GitHub's answer read), UpdateCheck, UpdateState, the app bundle of an executable,
                                 release notes embedded in the binary, WhatsNew; port: UpdateSteps
src/update/application/          UpdateRun: one check or one install at a time, each on its own thread
src/update/infrastructure/       GithubReleases (`ureq`), update staging (`ditto`, `codesign`), app bundle swap, relaunch (`open`), AppUpdate
src/update/ui/                   update strip, Updates dialog, What's new, running version button, release notes text (`egui_commonmark`)
assets/brand/                    logo and app icon (SVG), `icon.icns` copied into the app bundle
assets/fonts/                    IBM Plex Sans (OFL), embedded in the binary
tests/it/main.rs                 single integration binary
tests/it/ui_*.rs                 UI e2e
tests/it/gpu.rs, golden.rs       headless engine, golden compare
tests/fixtures/, tests/golden/   input files, reference renders
```

Modules are added by the milestone that needs them, not ahead of it.

## 4. Threads
- **UI thread**: egui frames, engine renders at screen resolution, and one
  reduced render read back per change of the development for the histogram
  (about 4 ms, `specs/histogram.md` rule 8); nothing else.
- **Preview loader** (`PhotoLoader`, one worker, `Backlog::LoadNewestOnly`): in
  Cull mode, reads the photo details and uploads the picture a RAW embeds, or
  the decoded photo when there is none. A RAW's picture is then replaced by the
  RAW the viewed-photo loader develops.
- **Photos ahead** (`PhotosAhead`, one worker): in Cull mode, develops the
  shown photos just after and just before the previewed one and keeps them on
  the GPU; a request no longer wanted when its turn comes is skipped. At most
  two photos are kept besides the previewed one.
- **Viewed-photo loader** (`PhotoLoader`, one worker, `Backlog::LoadNewestOnly`):
  decodes a file and uploads it to the GPU (`Engine::upload`; `wgpu::Device` and
  `Queue` are shared across threads). Requests made obsolete while it was busy
  are skipped, so fast navigation decodes only where the user stopped.
- **Thumbnail loader** (`PhotoLoader`, one worker, `Backlog::LoadAll`): one
  thumbnail per photo of the session, in order; replaced by a new one when
  another session is opened.
- **Cover loader** (`SeriesCovers`, one worker, `Backlog::LoadAll`): the first
  thumbnails of every series, for the sidebar. Both thumbnail workers read the
  thumbnail cache before decoding a photo.
- **Export worker** (`ExportRun`, one thread per export): decodes, uploads,
  renders (`Engine::render_pixels`, strip by strip), encodes and writes each
  photo. Display-stage renders are serialized by a lock: their uniforms are shared.
- **Update** (`UpdateRun`, one thread per check or install): asks GitHub for the
  latest release at launch; on request downloads, unpacks, validates and swaps
  the app bundle. The UI thread then saves and relaunches.
- Each result wakes the UI with `request_repaint`. A decoder panic fails that
  photo only.
- **Zone detection** (`DetectionRun`, one thread per detection): renders the
  photo for the model, downloads the model when missing, runs it on the CPU.
  Viewing another photo abandons it; the models are unloaded when it ends.
- **Enhancement** (`EnhancementRun`, one at a time): decodes the photo, runs the
  model tile by tile on the GPU, writes the enhancement file, uploads the result
  onto the photo on screen. It goes on while other photos are viewed; Cancel
  stops it at the next tile.
