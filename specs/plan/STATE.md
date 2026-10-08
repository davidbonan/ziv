# Progress state — ziv

> Source of truth for progress. Conventions: [`README.md`](README.md).
> `☐` to do · `◐` in progress · `☑` done+verified · `⊘` blocked · `⏭` deferred.

## Next actions
1. M10.1, M10.3 — check by hand in the real app that the mode (K2) and the
   place of the grid separator (K6) survive a relaunch: both ride on `eframe`
   storage, which a headless run does not have. Everything else of M10 is
   verified.
2. M13.7 — check by hand in a bundled app what a headless run cannot reach:
   check at launch, strip, install and relaunch, seen version, What's new.
   Needs a published release (M13.9). Unbundled behavior is verified.
3. M13.8 — `/release` is written, never run: its first run is M13.9.

## Blockers
None.

---

## ☑ M0 — Foundations · 3/3
- ☑ **M0.1 — Repository.** Git, private GitHub repo, toolchain pinned.
- ☑ **M0.2 — App skeleton + test loop.** lib + bin, `ZivApp` on eframe/wgpu,
  one UI e2e, whole-app headless capture proven. *Tests*: Eu, HV.
- ☑ **M0.3 — Working method.** Specs, plan, ADR 0001, Claude Code skills.

## ☑ M1 — Import & viewport · 9/9
Open a RAW or a standard image and see it, zoom and pan. Features F1.
*Spec*: `specs/import.md`. *Demo*: open a folder, browse the filmstrip, zoom
into the A7 IV RAW.

- ☑ **M1.1 — Color math.** Rec.709 ↔ Rec.2020 matrices and sRGB transfer
  functions as pure domain functions (ADR 0002). *Spec*: `specs/import.md` §4
  rule 24. *Tests*: U.
- ☑ **M1.2 — Standard image decoding.** JPEG / PNG / TIFF file → upright
  working-space image. *Spec*: §4 rules 14, 24 · A6, A7. *Depends*: M1.1.
  *Tests*: U, Eb.
- ☑ **M1.3 — Engine display render.** Working image → display-encoded texture on
  a `wgpu::Device`, offscreen; golden harness. *Spec*: A8. *Depends*: M1.1.
  *Tests*: G.
- ☑ **M1.4 — Viewport at fit.** The app shows a photo injected through the
  constructor seam, at fit, upright. *Spec*: §4 rules 8, 14 · A15.
  *Depends*: M1.2, M1.3. *Tests*: Eu, HV.
- ☑ **M1.5 — RAW decoding.** RAW file → upright working-space image, camera
  white balance and calibration; A7 IV verified. *Spec*: §4 rule 25 · A5.
  *Depends*: M1.1. *Tests*: U, Eb.
- ☑ **M1.6 — Background loading.** Decoding off the UI thread; loading and
  error states in the viewport. *Spec*: §4 rules 15, 16 · A10, A11.
  *Depends*: M1.4, M1.5. *Tests*: U, Eu, HV.
- ☑ **M1.7 — Zoom & pan.** Pointer-anchored zoom, fit / 100 % toggle, clamped
  pan, shortcuts. *Spec*: §4 rules 17–23 · A12–A14. *Depends*: M1.4.
  *Tests*: U, Eu, HV.
- ☑ **M1.8 — Opening a session.** Picker and drop → folder scan → session,
  first photo selected. *Spec*: §4 rules 1–8 · A1–A4. *Depends*: M1.6.
  *Tests*: U, Eb, Eu.
- ☑ **M1.9 — Filmstrip.** Thumbnails with their states, click and arrow-key
  selection. *Spec*: §4 rules 9–13 · A9, A10, A15. *Depends*: M1.8.
  *Tests*: Eu, HV.

## ☑ M2 — Global development · 11/11
Non-destructive adjustments with live preview, edits persisted. Feature F2.
*Spec*: `specs/develop.md`. *Demo*: develop the A7 IV RAW with every slider,
undo, compare with Before, quit, reopen and find the edit.

- ☑ **M2.1 — Exposure end to end.** Edit model, one engine stage, a develop
  panel with the Exposure slider, live preview. *Spec*: `specs/develop.md` §4
  rules 1, 3, 12 · D1, D3, D7. *Tests*: U, G, Eu, HV.
- ☑ **M2.2 — Design system.** Theme, spacing and type scales, slider component
  (centered fill, states, typed value, double-click reset), panel layout; M1's
  screens restyled. *Spec*: §5 · §4 rules 13, 14, 16 · D8, D16.
  *Depends*: M2.1. *Tests*: Eu, HV.
- ☑ **M2.3 — Base rendering for RAW.** Untouched RAW close to the camera JPEG.
  *Spec*: rules 9, 10 · D5, D6. *Depends*: M2.1. *Tests*: U, G, Eb.
- ☑ **M2.4 — Tone.** Contrast, highlights, shadows, whites, blacks.
  *Spec*: rules 4–6, 8 · D2, D3, D15. *Depends*: M2.2, M2.3. *Tests*: U, G, Eu.
- ☑ **M2.5 — White balance.** Temp and tint, Kelvin on RAW with as-shot
  defaults, relative on standard images. *Spec*: rule 2 · D2, D4.
  *Depends*: M2.2. *Tests*: U, G, Eu, Eb.
- ☑ **M2.6 — Presence.** Vibrance and saturation. *Spec*: rule 7 · D2, D3.
  *Depends*: M2.2. *Tests*: U, G, Eu.
- ☑ **M2.7 — Reset and panel states.** Reset of the whole edit; panel disabled
  while loading and on failure. *Spec*: rules 11, 15, 17 · D8, D9.
  *Depends*: M2.4–M2.6. *Tests*: Eu, HV.
- ☑ **M2.8 — Sidecar persistence.** Autosave, reload on selection, removal at
  all-default, write failure, unreadable or newer sidecar (ADR 0003).
  *Spec*: rules 18–22 · D9–D11. *Depends*: M2.7. *Tests*: U, Eb, Eu, HV.
- ☑ **M2.9 — Undo / redo.** Per-photo history. *Spec*: rules 23, 24 · D12.
  *Depends*: M2.7. *Tests*: U, Eu.
- ☑ **M2.10 — Before / after.** *Spec*: rules 25, 26 · D13. *Depends*: M2.7.
  *Tests*: Eu, HV.
- ☑ **M2.11 — Copy / paste.** *Spec*: rules 27–29 · D14. *Depends*: M2.9.
  *Tests*: U, Eu.

## ☑ M3 — Export · 5/5
Developed photos written to JPEG or PNG files. Feature F6.
*Spec*: `specs/export.md`. *Demo*: develop the A7 IV RAW, export it at a long
edge of 2048 as JPEG, then export the whole session; quit, relaunch and find
the export settings again.

- ☑ **M3.1 — Full-resolution render to pixels.** Display stage rendered in
  strips at any output size, assembled on the CPU. *Spec*: `specs/export.md`
  §4 rules 8, 10 · X1, X2. *Tests*: U, Eb.
- ☑ **M3.2 — Export of one photo to a file.** Size, naming without overwrite,
  JPEG and PNG encoding. *Spec*: rules 3, 10–12 · X3, X4. *Depends*: M3.1.
  *Tests*: U, Eb.
- ☑ **M3.3 — Background export of several photos.** Worker, stored edits,
  progress, cancel, skipped failures, result. *Spec*: rules 9, 13–16 · X5, X8.
  *Depends*: M3.2. *Tests*: U, Eb.
- ☑ **M3.4 — Export dialog and settings.** Scope, settings, destination,
  remembered between launches. *Spec*: rules 1–7 · X6, X7. *Depends*: M2.2.
  *Tests*: U, Eu.
- ☑ **M3.5 — Export in the app.** Shortcuts, button, progress and result in the
  window. *Spec*: rules 1, 7, 13–16 · X8, X9. *Depends*: M3.3, M3.4.
  *Tests*: HV.

## ☑ M4 — Local adjustments · 8/8
Brush, gradients and shapes as masks carrying their own adjustments. Feature F3.
*Spec*: `specs/local-adjustments.md` (ADR 0005). *Demo*: on the A7 IV RAW,
darken the sky with a linear gradient, brighten a face with a radial gradient,
paint with the brush; quit, relaunch, find the masks again; export.

- ☑ **M4.1 — Masks in the engine, linear gradient.** Mask model in the edit,
  coverage of a linear gradient, blend of local adjustments in the display
  stage, hidden and inverted masks, overlap. *Spec*: rules 4–9, 11 · L1–L3.
  *Tests*: U, G.
- ☑ **M4.2 — Masks section and local adjustments panel.** Add, list, select,
  hide, delete, limit of 16; the panel edits the selected mask. *Spec*: rules
  1–6 · L4, L5. *Depends*: M4.1. *Tests*: Eu.
- ☑ **M4.3 — Drawing on the photo and overlay.** Armed tool, linear gradient
  drawn and edited by its handles, `Space` to pan, red overlay and `O`.
  *Spec*: rules 10, 11, 16–20 · L6, L7. *Depends*: M4.2. *Tests*: U, G, Eu, HV.
- ☑ **M4.4 — Radial gradient.** *Spec*: rule 12 · L1, L6. *Depends*: M4.3.
  *Tests*: U, G, Eu.
- ☑ **M4.5 — Rectangle and polygon.** *Spec*: rules 13, 14 · L1, L6.
  *Depends*: M4.3. *Tests*: U, G, Eu.
- ☑ **M4.6 — Brush.** Strokes, erase, size, feather, flow, pointer circle.
  *Spec*: rule 15 · L1, L6. *Depends*: M4.3. *Tests*: U, G, Eu, HV.
- ☑ **M4.7 — Masks with the rest of the app.** Sidecar version 2, undo and
  redo, Before, Reset, paste, export. *Spec*: rules 21–25 · L8–L10.
  *Depends*: M4.4–M4.6. *Tests*: U, Eb, Eu.
- ☑ **M4.8 — Demo.** *Spec*: L11. *Depends*: M4.7. *Tests*: HV.

## ☑ M5 — Zone masks · 8/8
Masks generated on precise zones (hair, eyes, skin…). Feature F4.
*Spec*: `specs/zone-masks.md` (ADR 0006, 0007). *Demo*: on a photo of several
persons, mask the subject, the sky and the skin of one person; reopen; export.

- ☑ **M5.1 — Zone mask in the edit and the engine.** Coverage image as a mask
  shape, rendered as a layer, stored in the sidecar. *Spec*: rules 14–16 ·
  Z3, Z4. *Tests*: U, G, Eb.
- ☑ **M5.2 — Model store.** Download at first use with progress, checksum,
  reuse, failure. *Spec*: rules 11, 12 · Z1. *Tests*: U, Eb.
- ☑ **M5.3 — Subject detection.** Runtime, photo rendered for the model,
  BiRefNet to a coverage image. *Spec*: rules 2, 13 · Z2. *Depends*: M5.1,
  M5.2. *Tests*: U, Eb.
- ☑ **M5.4 — Subject and Background in the app.** Zone tools, background
  detection, messages, abandon. *Spec*: rules 1–6 · Z5, Z6. *Depends*: M5.3.
  *Tests*: U, Eu, HV.
- ☑ **M5.5 — Sky.** *Spec*: rules 2, 5 · Z7. *Depends*: M5.4. *Tests*: U, Eb.
- ☑ **M5.6 — Persons and their parts.** Person boxes, parts per person, faces.
  *Spec*: rule 8 · Z9. *Depends*: M5.3. *Tests*: U, Eb.
- ☑ **M5.7 — People picker.** *Spec*: rules 7–10 · Z8. *Depends*: M5.4, M5.6.
  *Tests*: U, Eu, HV.
- ☑ **M5.8 — Demo.** *Spec*: Z10. *Depends*: M5.5, M5.7. *Tests*: HV.

## ☑ M6 — Enhance · 6/6
Noise removed and detail strengthened by a model. Feature F5.
*Spec*: `specs/enhance.md` (ADR 0008, 0009, 0010). *Demo*: enhance a photo, dose the
intensity, reopen, export.

- ☑ **M6.1 — Models as a module of their own.** Store, runner and downloads
  move out of `zones`. *Spec*: rule 4. *Tests*: U.
- ☑ **M6.2 — Enhancement of an image.** Encoding, tiles, blend, unsharp mask.
  *Spec*: rule 2 · E1. *Depends*: M6.1. *Tests*: U.
- ☑ **M6.3 — Enhancement file.** *Spec*: rules 11, 12 · E2. *Depends*: M6.2.
  *Tests*: U, Eb.
- ☑ **M6.4 — Intensity in the edit and the engine.** Second source, mix in the
  display stage, export. *Spec*: rules 7–10, 13 · E3, E4. *Tests*: U, G, Eb.
- ☑ **M6.5 — Enhance in the app.** Detail section, run with progress and
  Cancel. *Spec*: rules 1–6 · E5, E6. *Depends*: M6.3, M6.4. *Tests*: U, Eu, HV.
- ☑ **M6.6 — Demo.** The model runs on the GPU (ADR 0010): a 32.7 MP RAW in
  about 20 s. *Spec*: E7. *Depends*: M6.5. *Tests*: HV.

## ☑ M7 — Series catalog · 9/9
Imports kept as series in a sidebar; the app reopens where it was left.
Feature F7. *Spec*: `specs/catalog.md` (ADR 0011). *Demo*: import a folder and a
set of files, edit a photo, quit, relaunch and find both series, the open one,
the photo and its edit.

- ☑ **M7.1 — Catalog end to end.** Series in the domain, the catalog file, an
  import stored as a series, the last series and its photo reopened at launch.
  *Spec*: `specs/catalog.md` rules 1–3, 13–15 · C1, C2, C4, C5. *Tests*: U, Eb, HV.
- ☑ **M7.2 — Sidebar.** Series listed most recent first with name and date,
  click to open on the remembered photo, Import… button, hide and show.
  *Spec*: rules 6, 8–11 · C6, C7. *Depends*: M7.1. *Tests*: Eu, HV.
- ☑ **M7.3 — Folder imported again.** New photos join the folder's series.
  *Spec*: rules 4, 5 · C3. *Depends*: M7.1. *Tests*: U, Eb.
- ☑ **M7.4 — Edited count.** "N of M edited" from the sidecars, following the
  edits of the open series. *Spec*: rules 7, 12 · C8. *Depends*: M7.2.
  *Tests*: U, Eb, Eu.
- ☑ **M7.5 — Thumbnail cache and series cover.** Thumbnails kept on disk, read
  back without decoding, refreshed when the photo changed; cover in the row.
  *Spec*: rules 7, 16–19, 33 · C6, C9. *Depends*: M7.2. *Tests*: U, Eb, Eu.
- ☑ **M7.6 — Missing photos and Locate.** Photos and series marked as not
  found; Locate on a missing series. *Spec*: rules 20–25 · C10, C11.
  *Depends*: M7.2. *Tests*: U, Eb, Eu.
- ☑ **M7.7 — Managing a series.** Row menu: Rename, Show in Finder, Remove.
  *Spec*: rules 26–30 · C12, C13. *Depends*: M7.2. *Tests*: U, Eu.
- ☑ **M7.8 — Catalog failures.** Unreadable or newer catalog, write failure.
  *Spec*: rules 31, 32 · C14. *Depends*: M7.1. *Tests*: U, Eb, Eu.
- ☑ **M7.9 — Demo.** *Spec*: C15. *Depends*: M7.3–M7.8. *Tests*: HV.

## ☑ M8 — Window shell · 8/8
The layout and the controls of direction A of the mockup.
*Spec*: `specs/shell.md`. *Demo*: the real app on a series, beside
`specs/mockups/shell.html`.

- ☑ **M8.1 — Top bar and layout.** Top bar with the sidebar button, the names,
  Before and Export…; sidebar and develop panel down to the bottom; a filmstrip
  without buttons. *Spec*: `specs/shell.md` rules 1–3, 5–8, 10 · S1, S3, S4.
  *Tests*: Eu, HV.
- ☑ **M8.2 — Zoom readout.** *Spec*: rule 4 · S2. *Depends*: M8.1. *Tests*: U, Eu.
- ☑ **M8.3 — Filmstrip position and edited marker.** *Spec*: rules 8, 9 · S5.
  *Depends*: M8.1. *Tests*: Eu, HV.
- ☑ **M8.4 — Mask tools as icons.** *Spec*: rules 11, 12 · S6. *Tests*: Eu, HV.
- ☑ **M8.5 — Folding sections.** Titles that fold, mask count on Masks.
  *Spec*: rules 13–15 · S7. *Depends*: M8.4. *Tests*: Eu, HV.
- ☑ **M8.6 — Panel foot.** Copy, Paste, Reset at the bottom; no panel title.
  *Spec*: rules 16, 17 · S8. *Depends*: M8.1. *Tests*: Eu.
- ☑ **M8.7 — Colored Temp and Tint tracks.** *Spec*: rules 18–20 · S9.
  *Tests*: Eu, HV.
- ☑ **M8.8 — Demo.** *Spec*: S10. *Depends*: M8.1–M8.7. *Tests*: HV.

## ☑ M9 — Histogram and shooting data · 5/5
The histogram of the developed photo and its shooting data at the top of the
develop panel. Feature F9. *Spec*: `specs/histogram.md`. *Demo*: the real app on
a series, the histogram following an edit, the shooting data under it.

- ☑ **M9.1 — Histogram of a developed photo.** Levels counted from display
  pixels, heights, the photo rendered small by the engine. *Spec*:
  `specs/histogram.md` rules 2–4, 8 · H1, H2, H5. *Tests*: U, Eb.
- ☑ **M9.2 — Histogram in the develop panel.** Drawn above the tool bar,
  following the edit and Before. *Spec*: rules 1, 5–7, 13 · H3, H4.
  *Depends*: M9.1. *Tests*: Eu, HV.
- ☑ **M9.3 — Shooting data of a file.** Read from RAW, JPEG, PNG and TIFF,
  worded. *Spec*: rules 10–12 · H6, H7. *Tests*: U, Eb.
- ☑ **M9.4 — Shooting data in the develop panel.** The line under the
  histogram. *Spec*: rule 9 · H8. *Depends*: M9.2, M9.3. *Tests*: Eu, HV.
- ☑ **M9.5 — Demo.** *Spec*: H9. *Depends*: M9.4. *Tests*: HV.

## ◐ M10 — Culling · 9/11
Sort a series in ziv instead of Bridge: grid and fast preview, ratings, rejected
mark, series filter, removal and Trash. Feature F10. *Spec*: `specs/culling.md` (ADR 0012, 0013).
*Demo*: import a folder, rate and reject in Cull mode, trash the rejected,
filter at 2 stars or more, develop only the kept photos; quit, relaunch and
find marks, filter and mode again.

- ◐ **M10.1 — Cull mode end to end.** Mode switch in the top bar, `G` and `D`,
  mode remembered; grid of the open series with click and arrow selection,
  beside the preview of the selected photo at fit. *Spec*: `specs/culling.md`
  rules 1–11, 16, 20 · K1, K2, K3. *Tests*: U, Eu, HV.
- ☑ **M10.2 — Embedded preview.** The preview of a RAW comes from its embedded
  picture without decoding the sensor data; fallback to the unedited
  development; zoom and pan. *Spec*: rules 16–19 · K7, K8. *Depends*: M10.1.
  *Tests*: U, Eb, Eu.
- ◐ **M10.3 — Photo details and separator.** Name, shooting data, pixel size,
  date, file size under the preview; resizable separator, remembered.
  *Spec*: rules 15, 21, 22 · K6, K9. *Depends*: M10.1. *Tests*: U, Eb, Eu, HV.
- ☑ **M10.4 — Selection of several photos.** `Cmd`+click, `Shift`+click,
  `Cmd+A`, the count in the details; double click and `Enter` to Develop.
  *Spec*: rules 12–14, 23 · K4, K5, K9. *Depends*: M10.1. *Tests*: U, Eu.
- ☑ **M10.5 — Marks.** Ratings and rejected mark by keys and stars, on the
  selection, in both modes, shown in grid, filmstrip and details, kept between
  launches. *Spec*: rules 7, 24–29 · K10, K11. *Depends*: M10.4.
  *Tests*: U, Eb, Eu.
- ☑ **M10.6 — Series filter.** Filter in the top bar, applied to grid and
  filmstrip, position counting shown photos, selection following, empty
  result, remembered per series. *Spec*: rules 30–37 · K12, K13, K14.
  *Depends*: M10.5. *Tests*: U, Eb, Eu.
- ☑ **M10.7 — Remove from series.** `Delete` and the cell menu; neighbor
  selected; emptied series leaves the catalog. *Spec*: rules 38, 41, 42 · K15.
  *Depends*: M10.4. *Tests*: U, Eb, Eu.
- ☑ **M10.8 — Move to Trash.** Confirmation, photo and the files beside it to
  the Trash, failures counted, missing in other series. *Spec*: rules 39,
  41–44 · K16, K17. *Depends*: M10.7. *Tests*: U, Eb, Eu.
- ☑ **M10.9 — Trash rejected.** *Spec*: rule 40 · K16. *Depends*: M10.5, M10.8.
  *Tests*: U, Eu.
- ☑ **M10.11 — Photos developed ahead.** The shown photos after and before
  the selected one are developed before they are selected. *Spec*: rule 45 ·
  K19. *Depends*: M10.2. *Tests*: U, HV.
- ☑ **M10.10 — Demo.** *Spec*: K18. *Depends*: M10.2, M10.3, M10.6, M10.9.
  *Tests*: HV.

## ☑ M11 — Tone curve and color · 10/10
Point curve, color mixer and color grading on the photo. Feature F2.
*Spec*: `specs/curve-and-color.md` (ADR 0014). *Demo*: on the A7 IV RAW, an
S-curve, lifted blue blacks, greens desaturated, blues darkened, teal shadows
and orange highlights; quit, relaunch and find the edit again; export.

- ☑ **M11.1 — RGB curve end to end.** Curve in the edit, its engine stage, the
  Tone curve section with a graph where points are added, moved and removed;
  live preview. *Spec*: `specs/curve-and-color.md` rules 1, 3, 5, 7–11 ·
  T1, T2, T3, T4. *Tests*: U, G, Eu, HV.
- ☑ **M11.2 — Channel curves.** Red, Green and Blue curves, the channel
  selector and its marks, Reset curve. *Spec*: rules 6, 10, 12 · T3, T5.
  *Depends*: M11.1. *Tests*: U, G, Eu.
- ☑ **M11.3 — Histogram behind the graph.** *Spec*: rule 13 · T6.
  *Depends*: M11.1. *Tests*: Eu, HV.
- ☑ **M11.4 — Color mixer in the engine.** Hue, Saturation and Luminance of
  the eight color ranges in the edit and the display stage. *Spec*: rules
  15–18 · T1, T7, T8. *Tests*: U, G.
- ☑ **M11.5 — Color mixer section.** Hue / Saturation / Luminance selector,
  eight sliders with colored tracks, marks. *Spec*: rules 14, 19, 20 · T9.
  *Depends*: M11.4. *Tests*: Eu, HV.
- ☑ **M11.6 — Color grading in the engine.** Four tonal zones, Blending and
  Balance in the edit and the display stage. *Spec*: rules 26–30 · T1, T10,
  T11. *Tests*: U, G.
- ☑ **M11.7 — Color wheel and zone views.** The wheel component; Shadows,
  Midtones, Highlights and Global views with their sliders, Blending and
  Balance, the selector and its marks. *Spec*: rules 21, 23–25, 31 · T12, T13.
  *Depends*: M11.6. *Tests*: Eu, HV.
- ☑ **M11.8 — 3-way view.** Three wheels together with their Luminance.
  *Spec*: rule 22 · T13. *Depends*: M11.7. *Tests*: Eu, HV.
- ☑ **M11.9 — The three tools with the rest of the app.** Section order and
  masks, sidecar version, undo and redo, Before, Reset, paste, export, order
  independence. *Spec*: rules 1, 2, 4, 32–36 · T14–T18. *Depends*: M11.2,
  M11.5, M11.8. *Tests*: U, G, Eb, Eu.
- ☑ **M11.10 — Demo.** *Spec*: rule 37 · T19. *Depends*: M11.3, M11.9.
  *Tests*: HV.

## ☑ M12 — Crop and straighten · 8/8
Reframe a photo: crop frame, aspect ratios, angle, quarter turns, mirror.
Feature F11. *Spec*: `specs/crop-and-straighten.md` (ADR 0015). *Demo*: on the A7 IV RAW,
level the horizon with the level tool, crop to 4 : 5 portrait, turn a photo
shot sideways, draw a gradient on the framed photo; quit, relaunch and find
the framing again; export the framed file.

- ☑ **M12.1 — Crop frame end to end.** Framing in the edit, the engine
  rendering the framed photo, the viewport on it; crop mode with its button
  and `R`, the frame with handles, dimmed outside and thirds, free resize and
  move, Done and `Esc`. *Spec*: `specs/crop-and-straighten.md` rules 1–13,
  25, 26 · F1–F5, F13. *Tests*: U, G, Eu, HV.
- ☑ **M12.2 — Aspect ratios.** Ratio selector, lock, Custom, `X`. *Spec*:
  rules 14–17 · F6, F7. *Depends*: M12.1. *Tests*: U, Eu.
- ☑ **M12.3 — Straighten angle.** Angle in the edit and the engine, the
  slider, the drag outside the frame, the frame held inside the picture.
  *Spec*: rules 18–20, 31 · F8, F9, F10. *Depends*: M12.1. *Tests*: U, G, Eu.
- ☑ **M12.4 — Level tool.** *Spec*: rule 21 · F11. *Depends*: M12.3.
  *Tests*: U, Eu.
- ☑ **M12.5 — Quarter turns and mirror.** In the edit and the engine, the
  four buttons, `Cmd+[` and `Cmd+]`; frame, angle and masks carried along.
  *Spec*: rules 22–24 · F12. *Depends*: M12.3. *Tests*: U, G, Eu.
- ☑ **M12.6 — Masks, zones and enhancement under a framing.** Masks kept on
  the picture and drawn on the framed photo; zone detection and enhancement on
  the whole picture. *Spec*: rules 27–29 · F14, F15. *Depends*: M12.5.
  *Tests*: U, G, Eb.
- ☑ **M12.7 — Framing with the rest of the app.** Sidecar version, edited
  marker, undo and redo, Before, Reset crop, Reset, paste, histogram, export.
  *Spec*: rules 30, 32–39 · F16–F19. *Depends*: M12.5. *Tests*: U, Eb, Eu.
- ☑ **M12.8 — Demo.** *Spec*: rule 40 · F20. *Depends*: M12.2, M12.4, M12.6,
  M12.7. *Tests*: HV.

## ◐ M13 — Distribution and update · 6/9
ziv installed as an application, updating itself from GitHub Releases, showing
its release notes. Feature F12. *Spec*: `specs/update.md` (ADR 0016). *Demo*:
publish two versions with `/release`, install the first with the install
script, let it find the second, install it from the app and read What's new.

- ☑ **M13.1 — Package and release pipeline.** Bundle script, install script,
  release workflow on a version tag, README. *Spec*: `specs/update.md` rules
  1–3 · P1–P3. *Tests*: run of the bundle script.
- ☑ **M13.2 — Update check.** Version, release answer, check against GitHub's
  latest release. *Spec*: rules 6, 7 · P4–P6. *Tests*: U, Eb.
- ☑ **M13.3 — Update install.** App bundle of the executable; download,
  unpack, signature validation, swap with rollback, relaunch. *Spec*: rules
  8–10 · P7–P10. *Tests*: U, Eb.
- ☑ **M13.4 — Update run.** States, worker thread, one operation at a time,
  silent failure at launch. *Spec*: rules 4, 5, 12 · P11. *Depends*: M13.2,
  M13.3. *Tests*: U.
- ☑ **M13.5 — Release notes and What's new decision.** Notes embedded in the
  binary, seen version. *Spec*: rules 19–23 · P15, P16. *Depends*: M13.2.
  *Tests*: U.
- ☑ **M13.6 — Update strip, Updates dialog, What's new window, version in the
  sidebar.** *Spec*: rules 14–18, 21 · P12–P14, P17. *Depends*: M13.4, M13.5.
  *Tests*: Eu.
- ◐ **M13.7 — Update in the app.** Check at launch, strip, dialog, install and
  relaunch with everything saved, seen version remembered, no install during
  an export or an enhancement. *Spec*: rules 4, 8, 11, 13, 21–24 · P18.
  *Depends*: M13.6. *Tests*: HV.
- ◐ **M13.8 — `/release` skill.** *Spec*: rule 25. *Depends*: M13.1, M13.5.
  *Tests*: read in review.
- ◐ **M13.9 — Demo.** *Spec*: P19. *Depends*: M13.7, M13.8. *Tests*: by hand.

## ☑ M14 — AI presets · 10/10
One-click retouches: a preset detects its zones, masks them and sets their
adjustments, dosed by one Intensity. Feature F13. *Spec*: `specs/ai-presets.md`.
*Demo*: on a portrait and a landscape, Whiter teeth, Bright eyes, Enhanced sky
and Subject pop; dose one with Intensity; quit, relaunch and find them again;
export.

- ☑ **M14.1 — Enhanced sky end to end.** Preset and applied preset in the
  edit, the Presets section with its button, detection and message, the
  applied preset listed with its mask and selected. *Spec*:
  `specs/ai-presets.md` rules 1–5, 7, 8, 10 · R1, R2. *Tests*: U, Eu, HV.
- ☑ **M14.2 — Intensity.** In the edit and the engine; the panel of a selected
  applied preset with its slider. *Spec*: rules 11–13 · R3, R4.
  *Depends*: M14.1. *Tests*: U, G, Eu.
- ☑ **M14.3 — Subject pop.** Two masks or none; disabled when fewer than two
  masks are left. *Spec*: rules 2, 5, 7 · R2, R5. *Depends*: M14.1.
  *Tests*: U, Eu.
- ☑ **M14.4 — Bright eyes.** The eyes of every person in one mask, without a
  picker. *Spec*: rules 5–7 · R6. *Depends*: M14.1. *Tests*: U, Eu, Eb.
- ☑ **M14.5 — Teeth zone.** Teeth of every person as a coverage image.
  *Spec*: rules 5–7 · R7. *Tests*: U, Eb.
- ☑ **M14.6 — Whiter teeth.** *Spec*: rules 5–7 · R7. *Depends*: M14.4,
  M14.5. *Tests*: U, Eu.
- ☑ **M14.7 — Applied presets with the rest of the app.** Deleting a mask or
  the applied preset, applying twice, sidecar version, undo and redo, Before,
  Reset, paste, export. *Spec*: rules 9, 14–19 · R8–R10. *Depends*: M14.2,
  M14.3. *Tests*: U, Eb, Eu.
- ☑ **M14.8 — Demo.** *Spec*: R11. *Depends*: M14.6, M14.7. *Tests*: HV.
- ☑ **M14.9 — More presets.** Skin glow, Lip color, Hair shine, Dramatic sky,
  Golden sky. *Spec*: rules 5, 6 · R12. *Depends*: M14.4. *Tests*: U, HV.
- ☑ **M14.10 — Presets by group.** Two a row under Portrait and Scene.
  *Spec*: rule 1 · R12. *Depends*: M14.9. *Tests*: Eu, HV.
