# Progress state — ziv

> Source of truth for progress. Conventions: [`README.md`](README.md).
> `☐` to do · `◐` in progress · `☑` done+verified · `⊘` blocked · `⏭` deferred.

## Next actions
1. None planned: every milestone of the plan is done. Next: frame what comes after with `/spec`.

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
